use actix::prelude::*;
use actix::{Actor, Context, Handler, Message};
use fut::wrap_future;
use std::{
    collections::HashMap,
    fs::File as SyncFile,
    io::{self, BufRead},
    sync::Arc,
    time::Duration,
};
use tokio::task::JoinHandle;
use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::shop_connection::stock_requester::StockResult;
use tokio::fs;

const NOT_ENOUGH_STOCK_ERR: &str = "Not enough stock";
const NOT_AVAILABLE_FLAVOR_ERR: &str = "Flavor not available";
const INVALID_CANCEL_ERR: &str = "Tried to cancel non-existing reserve";
const INVALID_CONFIRM_ERR: &str = "Tried to confirm non-existing reserve";

/// Function simulating a delay corresponding to actually going
/// to get the ice cream from the container.
async fn go_to_ice_cream_container_for(amount: u32) {
    // U[0.75, 1.25]
    let rand_factor = (rand::random::<f32>() / 2.0) + 1.0;
    let delay = (amount as f32 * rand_factor) as u64;
    trace!("Going to get ice cream. Delay: {}", delay);
    tokio::time::sleep(Duration::from_millis(delay)).await;
}

/// Helper function to confirm a reserve amount.
async fn write_reserve_to_confirmed(
    reserved: Arc<Mutex<u32>>,
    amount: u32,
    confirmed: Arc<Mutex<u32>>,
) -> Result<(), String> {
    let mut reserved = reserved.lock().await;
    let mut confirmed = confirmed.lock().await;
    if amount > *reserved {
        error!("Received request to confirm more than reserved: {}. This REALLY should have not happened", amount);
        *reserved = 0;
        return Err("Invalid reserve".to_string());
    }
    *confirmed += amount;
    *reserved -= amount;
    Ok(())
}

/// Helper function to confirm a reserve directly to the main inventory
async fn write_reserve_to_stock(
    reserved: Arc<Mutex<u32>>,
    amount: u32,
    stocked: Arc<Mutex<u32>>,
) -> Result<(), String> {
    let mut stocked = stocked.lock().await;
    let mut reserved = reserved.lock().await;
    if amount > *reserved {
        error!("Received request to confirm more than reserved: {}. This REALLY should have not happened", amount);
        *reserved = 0;
        return Err(INVALID_CONFIRM_ERR.to_string());
    }
    *stocked -= amount;
    *reserved -= amount;
    Ok(())
}

/// Actor in charge of managing access to the stock.
/// During regular operation, all petitions are first reserved, and then confirmed
/// by the order solicitor once the order is complete or aborted in case the order couldn't be fulfilled.
/// When doing a backup, all changes are written first to a log while the backup is in process
/// to ensure no dirty data is written.
pub struct StockManager {
    inventory: HashMap<String, Arc<Mutex<u32>>>,
    reserved: HashMap<String, Arc<Mutex<u32>>>,
    confirmed: HashMap<String, Arc<Mutex<u32>>>,
    backup_in_process: bool,
    confirms_since_last_backup: usize,
    max_confirms_before_backup: usize,
}

impl StockManager {
    /// Creates a new manager, taking stock from the given file.
    /// The file must be in the format of `flavor,amount` per line.
    /// Panics if the file does not exist.
    pub fn new(initial_stock_path: &str, max_orders_before_backup: usize) -> Self {
        info!("Creating StockManager from: {}", initial_stock_path);
        let mut inventory = HashMap::new();
        let file = SyncFile::open(initial_stock_path).unwrap_or_else(|_| {
            error!("File {} not found. Aborting", initial_stock_path);
            panic!("File not found")
        });
        let reader = io::BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.split(',').collect();
            let item_name = parts[0];
            let item_count_result = parts[1].parse::<u32>();
            match item_count_result {
                Ok(item_count) => {
                    inventory.insert(
                        item_name.to_string().to_ascii_uppercase(),
                        Arc::new(Mutex::new(item_count)),
                    );
                }
                Err(_) => {
                    continue;
                }
            }
        }
        debug!("StockManager created with stock: {:?}", inventory);
        StockManager {
            inventory,
            reserved: HashMap::new(),
            confirmed: HashMap::new(),
            backup_in_process: false,
            confirms_since_last_backup: 0,
            max_confirms_before_backup: max_orders_before_backup,
        }
    }
}

impl Actor for StockManager {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        debug!("StockManager started");
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ReserveIceCream {
    pub requester_addr: Recipient<StockResult>,
    pub request_id: Uuid,
    pub flavor: String,
    pub amount: u32,
}

impl Handler<ReserveIceCream> for StockManager {
    type Result = ResponseFuture<Result<(), String>>;

    /// Asynchronously places a reserve for the given amount of the given flavor.
    /// Status will be notified to the given address, along with the given id to
    /// distinguish among requests belonging to different orders.
    ///
    /// Returns a future that will resolve to the result of the reserve operation.
    fn handle(&mut self, msg: ReserveIceCream, _: &mut Self::Context) -> Self::Result {
        trace!(
            "Received request of {} {} for {}",
            msg.amount,
            msg.flavor,
            msg.request_id.as_fields().0
        );
        let flavor = msg.flavor.to_ascii_uppercase();
        let amount = msg.amount;
        let req_addr = msg.requester_addr.clone();
        let req_idx = msg.request_id;
        let stock = self.inventory.get(&flavor);
        match stock {
            Some(stock) => {
                let stock = stock.clone();

                let reserved = self
                    .reserved
                    .entry(flavor.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(0)))
                    .clone();

                let to_confirm: Option<Arc<Mutex<u32>>> = if self.confirmed.contains_key(&flavor) {
                    Some(self.confirmed.get(&flavor).unwrap().clone())
                } else {
                    None
                };

                Box::pin(async move {
                    let stock = stock.lock().await;
                    let mut reserved = reserved.lock().await;
                    let mut total = *stock - *reserved;

                    if let Some(to_confirm) = to_confirm {
                        let to_confirm = to_confirm.lock().await;
                        total -= *to_confirm;
                    }

                    go_to_ice_cream_container_for(amount).await;

                    if total >= amount {
                        *reserved += amount;

                        trace!(
                            "Could reserve {} {} for {}",
                            msg.amount,
                            msg.flavor,
                            msg.request_id.as_fields().0
                        );
                        req_addr.do_send(StockResult {
                            requester: req_idx,
                            result: Some((flavor, amount)),
                        });
                        Ok(())
                    } else {
                        trace!(
                            "Could not get {} {} for {}",
                            msg.amount,
                            msg.flavor,
                            msg.request_id.as_fields().0
                        );
                        req_addr.do_send(StockResult {
                            requester: req_idx,
                            result: None,
                        });
                        Err(NOT_ENOUGH_STOCK_ERR.to_string())
                    }
                })
            }
            None => {
                info!(
                    "Received request for non-existent flavor: {}, for: {}",
                    msg.flavor,
                    msg.request_id.as_fields().0
                );
                req_addr.do_send(StockResult {
                    requester: req_idx,
                    result: None,
                });
                Box::pin(async { Err(NOT_AVAILABLE_FLAVOR_ERR.to_string()) })
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "Vec<JoinHandle<Result<(), String>>>")]
pub struct CancelReserve {
    pub reserves: Vec<(String, u32)>,
}

impl Handler<CancelReserve> for StockManager {
    type Result = Vec<JoinHandle<Result<(), String>>>;

    /// Asynchronously cancels reserves for the given amounts of the given flavors.
    ///
    /// Returns a vector of the handles of these operations.
    fn handle(&mut self, msg: CancelReserve, _: &mut Self::Context) -> Self::Result {
        trace!("Received request to put back {:?}", msg.reserves);
        let mut handles = Vec::new();
        for (flavor, amount) in msg.reserves {
            let reserved = self.reserved.get(&flavor);
            match reserved {
                Some(reserved) => {
                    let reserved = reserved.clone();
                    let handle: JoinHandle<Result<(), String>> = actix::spawn(async move {
                        go_to_ice_cream_container_for(amount).await;
                        let mut reserved = reserved.lock().await;
                        if amount > *reserved {
                            error!("Received request to put back more than reserved: {}. This REALLY should have not happened", flavor);
                            *reserved = 0;
                            return Err(INVALID_CANCEL_ERR.to_string());
                        }
                        *reserved -= amount;
                        Ok(())
                    });
                    handles.push(handle);
                }
                None => {
                    error!("Received request to put back non-existent flavor: {}. This REALLY should have not happened", flavor);
                    handles.push(actix::spawn(async { Err(INVALID_CANCEL_ERR.to_string()) }));
                }
            }
        }
        handles
    }
}

#[derive(Message)]
#[rtype(result = "Vec<JoinHandle<Result<(), String>>>")]
pub struct ConfirmReserve {
    pub reserves: Vec<(String, u32)>,
}

impl Handler<ConfirmReserve> for StockManager {
    type Result = Vec<JoinHandle<Result<(), String>>>;

    /// Asynchronously confirms previously reserved items, effectively commiting them to the main inventory.
    ///
    /// Returns a vector of the handles of these operations.
    fn handle(&mut self, msg: ConfirmReserve, ctx: &mut Self::Context) -> Self::Result {
        trace!("Received request to confirm {:?}", msg.reserves);
        let mut handles = Vec::new();
        for (flavor, amount) in msg.reserves {
            let item = self.reserved.get(&flavor);
            match item {
                Some(item) => {
                    if self.backup_in_process {
                        let item = item.clone();
                        let to_confirm = self
                            .confirmed
                            .get(&flavor)
                            .unwrap_or(&Arc::new(Mutex::new(0)))
                            .clone();
                        handles.push(actix::spawn(async move {
                            write_reserve_to_confirmed(item.clone(), amount, to_confirm.clone())
                                .await
                        }));
                    } else if let Some(stock) = self.inventory.get(&flavor) {
                        let item = item.clone();
                        let stock = stock.clone();
                        handles.push(actix::spawn(async move {
                            write_reserve_to_stock(item.clone(), amount, stock.clone()).await
                        }));
                    } else {
                        error!("Received request to confirm non-existent flavor: {}. This REALLY should have not happened", flavor);
                        handles.push(actix::spawn(async { Err(INVALID_CONFIRM_ERR.to_string()) }));
                    }
                }
                None => {
                    error!("Received request to confirm non-existent flavor: {}. This REALLY should have not happened", flavor);
                    handles.push(actix::spawn(async { Err(INVALID_CONFIRM_ERR.to_string()) }));
                }
            }
        }
        self.confirms_since_last_backup += 1;
        if self.confirms_since_last_backup >= self.max_confirms_before_backup
            && !self.backup_in_process
        {
            info!("Reached max confirms since last backup. Starting backup");
            ctx.address().do_send(BackupShop {
                out_dir: "backups".to_string(),
                out_file: format!(
                    "auto-{}.csv",
                    chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
                ),
            });
        }
        handles
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct BackupShop {
    pub out_dir: String,
    pub out_file: String,
}

impl Handler<BackupShop> for StockManager {
    type Result = ResponseFuture<Result<(), String>>;

    /// Starts the process of backing up the shop to the given file.
    /// On error, the resulting file will be left empty (truncated if it exists).
    ///
    /// Returns a future that will resolve to the result of the backup operation.
    fn handle(&mut self, msg: BackupShop, ctx: &mut Self::Context) -> Self::Result {
        if self.backup_in_process {
            warn!("Backup already in process. Ignoring request");
            return Box::pin(async { Err("Backup already in process".to_string()) });
        }

        info!("Starting shop backup");
        self.backup_in_process = true;
        self.confirms_since_last_backup = 0;
        trace!("Backup in process, now deferring writes");
        let out_file = msg.out_file.clone();
        let out_dir = msg.out_dir.clone();
        let inventory = self.inventory.clone();

        let stock_addr = ctx.address();
        Box::pin(async move {
            if let Err(e) = fs::create_dir_all(&out_dir).await {
                error!("Error creating output directory: {}", e);
                stock_addr.do_send(WriteConfirmedToInventory);
                return Err("Error creating output directory".to_string());
            }
            let filename = format!("{}/{}", out_dir, out_file);
            let file = File::create(filename.clone()).await;
            match file {
                Ok(mut file) => {
                    for (flavor, count) in inventory.iter() {
                        let count = count.lock().await;
                        if file
                            .write_all(format!("{},{}\n", flavor, *count).as_bytes())
                            .await
                            .is_err()
                        {
                            error!("Error writing to file. Aborting backup");
                            let _ = SyncFile::create(filename.clone());
                            break;
                        }
                    }
                    if file.flush().await.is_err() {
                        warn!("Failed to flush backup to disk. Consider doing another one");
                    }
                    info!("Writing shop backup completed. Output at: {}", filename);
                    stock_addr.do_send(WriteConfirmedToInventory);
                    Ok(())
                }
                Err(e) => {
                    error!("Error creating file: {}. Aborting backup", e);
                    stock_addr.do_send(WriteConfirmedToInventory);
                    Err("Error creating file".to_string())
                }
            }
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct WriteConfirmedToInventory;

impl Handler<WriteConfirmedToInventory> for StockManager {
    type Result = ();

    /// Writes all confirmed changes to the main inventory, finalizing the backup process.
    ///
    /// If there's an error when writing one of the flavors, it countinues with the rest
    fn handle(&mut self, _: WriteConfirmedToInventory, ctx: &mut Self::Context) -> Self::Result {
        self.backup_in_process = false;
        trace!("Backup process stopped. Now writing confirms to main inventory");
        for (flavor, amount) in self.confirmed.iter() {
            if let Some(stock) = self.inventory.get(flavor) {
                let stock = stock.clone();
                let amount = amount.clone();
                wrap_future::<_, Self>(async move {
                    let mut stock = stock.lock().await;
                    let mut sold = amount.lock().await;
                    *stock -= *sold;
                    *sold = 0;
                })
                .spawn(ctx);
            } else {
                warn!(
                    "There was a confirm for {}, which was not in the inventory",
                    flavor
                );
            }
        }
        trace!("Confirm writing finished. Everything in the main inventory now.");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const SEND_ERROR: &str = "Error sending message";

    fn testing_manager() -> StockManager {
        let mut inv = HashMap::new();
        inv.insert("VAINILLA".to_string(), Arc::new(Mutex::new(10)));
        inv.insert("CHOCOLATE".to_string(), Arc::new(Mutex::new(5)));
        inv.insert("FRUTILLA".to_string(), Arc::new(Mutex::new(3)));
        StockManager {
            inventory: inv,
            reserved: HashMap::new(),
            confirmed: HashMap::new(),
            backup_in_process: false,
            confirms_since_last_backup: 0,
            max_confirms_before_backup: 5,
        }
    }

    struct MockRequester {
        last_received_result: Option<StockResult>,
    }
    impl MockRequester {
        fn new() -> Self {
            MockRequester {
                last_received_result: None,
            }
        }
    }

    #[derive(Message)]
    #[rtype(result = "bool")]
    struct GotResult {
        result: StockResult,
    }
    impl Handler<GotResult> for MockRequester {
        type Result = bool;

        fn handle(&mut self, msg: GotResult, _: &mut Self::Context) -> Self::Result {
            println!("Stored result: {:?}", self.last_received_result);
            match &self.last_received_result {
                Some(last) => last == &msg.result,
                None => false,
            }
        }
    }

    impl Actor for MockRequester {
        type Context = Context<Self>;
    }
    impl Handler<StockResult> for MockRequester {
        type Result = Result<(), String>;

        fn handle(&mut self, msg: StockResult, _: &mut Self::Context) -> Self::Result {
            println!("Received result: {:?}", msg);
            self.last_received_result = Some(msg);
            Ok(())
        }
    }

    #[actix_rt::test]
    async fn test01_cannot_reserve_more_than_available() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let result = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 15,
            })
            .await
            .expect(SEND_ERROR);

        assert_eq!(Err(NOT_ENOUGH_STOCK_ERR.to_string()), result);
    }

    #[actix_rt::test]
    async fn test02_can_reserve_ice_cream() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let result = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 5,
            })
            .await
            .expect(SEND_ERROR);

        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test03_cannot_reserve_if_shop_runs_out() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let result_1 = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 9,
            })
            .await
            .expect(SEND_ERROR);

        let result_2 = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 5,
            })
            .await
            .expect(SEND_ERROR);

        assert!(result_1.is_ok());
        assert_eq!(Err(NOT_ENOUGH_STOCK_ERR.to_string()), result_2);
    }

    #[actix_rt::test]
    async fn test04_cannot_reserve_non_existent_flavor() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let result = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "PISTACHO".to_string(),
                amount: 5,
            })
            .await
            .expect(SEND_ERROR);

        assert_eq!(Err(NOT_AVAILABLE_FLAVOR_ERR.to_string()), result);
    }

    #[actix_rt::test]
    async fn test05_can_cancel_reserve_and_stock_becomes_available() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let _ = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 10,
            })
            .await
            .expect(SEND_ERROR);

        let cancel_result = stock
            .send(CancelReserve {
                reserves: vec![("VAINILLA".to_string(), 5)],
            })
            .await
            .expect(SEND_ERROR)
            .pop()
            .expect("Error getting handle from message result")
            .await
            .expect("Error getting result from handle");

        let reserve_result = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 5,
            })
            .await
            .expect(SEND_ERROR);

        assert!(cancel_result.is_ok());
        assert!(reserve_result.is_ok());
    }

    #[actix_rt::test]
    async fn test06_cannot_cancel_what_has_not_been_reserved() {
        let stock = testing_manager().start();

        let cancel_result = stock
            .send(CancelReserve {
                reserves: vec![("VAINILLA".to_string(), 5)],
            })
            .await
            .expect(SEND_ERROR)
            .pop()
            .expect("Error getting handle from message result")
            .await
            .expect("Error getting result from handle");

        assert_eq!(Err(INVALID_CANCEL_ERR.to_string()), cancel_result);
    }

    #[actix_rt::test]
    async fn test07_can_confirm_reserves() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let _ = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 10,
            })
            .await
            .expect(SEND_ERROR);

        let confirm_result = stock
            .send(ConfirmReserve {
                reserves: vec![("VAINILLA".to_string(), 10)],
            })
            .await
            .expect(SEND_ERROR)
            .pop()
            .expect("Error getting handle from message result")
            .await
            .expect("Error getting result from handle");

        assert!(confirm_result.is_ok());
    }

    #[actix_rt::test]
    async fn test08_cannot_confirm_before_reserve() {
        let stock = testing_manager().start();

        let confirm_result = stock
            .send(ConfirmReserve {
                reserves: vec![("VAINILLA".to_string(), 10)],
            })
            .await
            .expect(SEND_ERROR)
            .pop()
            .expect("Error getting handle from message result")
            .await
            .expect("Error getting result from handle");

        assert_eq!(Err(INVALID_CONFIRM_ERR.to_string()), confirm_result);
    }

    #[actix_rt::test]
    async fn test09_cannot_cancel_after_confirm() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let _ = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 10,
            })
            .await
            .expect(SEND_ERROR);

        let _ = stock
            .send(ConfirmReserve {
                reserves: vec![("VAINILLA".to_string(), 10)],
            })
            .await
            .expect(SEND_ERROR);

        let cancel_result = stock
            .send(CancelReserve {
                reserves: vec![("VAINILLA".to_string(), 5)],
            })
            .await
            .expect(SEND_ERROR)
            .pop()
            .expect("Error getting handle from message result")
            .await
            .expect("Error getting result from handle");

        assert_eq!(Err(INVALID_CANCEL_ERR.to_string()), cancel_result);
    }

    #[actix_rt::test]
    async fn test10_manager_responds_after_successful_reserve() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let _ = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 10,
            })
            .await
            .expect(SEND_ERROR);

        let result = requester
            .send(GotResult {
                result: StockResult {
                    requester: Uuid::nil(),
                    result: Some(("VAINILLA".to_string(), 10)),
                },
            })
            .await
            .expect(SEND_ERROR);

        assert!(result);
    }

    #[actix_rt::test]
    async fn test11_manager_responds_after_failed_request() {
        let stock = testing_manager().start();
        let requester = MockRequester::new().start();

        let _ = stock
            .send(ReserveIceCream {
                requester_addr: requester.clone().recipient(),
                request_id: Uuid::nil(),
                flavor: "VAINILLA".to_string(),
                amount: 15,
            })
            .await
            .expect(SEND_ERROR);

        let result = requester
            .send(GotResult {
                result: StockResult {
                    requester: Uuid::nil(),
                    result: None,
                },
            })
            .await
            .expect(SEND_ERROR);

        assert!(result);
    }
}
