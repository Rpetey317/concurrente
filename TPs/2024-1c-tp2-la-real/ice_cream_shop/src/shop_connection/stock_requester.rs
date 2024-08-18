use std::collections::HashMap;

use actix::prelude::*;
use tracing::{debug, info, trace};
use uuid::Uuid;

use crate::shop_connection::robot_communicator::SendMessage;
use crate::shop_stock::active_order::ActiveOrder;
use crate::shop_stock::stock_manager::{
    CancelReserve, ConfirmReserve, ReserveIceCream, StockManager,
};
use common::communication::shop_requests::ShopResponse;

use super::robot_communicator::RobotCommunicator;

/// Actor responsible for mediation between the stock manager and the robots.
/// Responsible for forwarding messages both ways
pub struct StockRequester {
    stock_manager: Addr<StockManager>,
    connections: HashMap<Uuid, Addr<RobotCommunicator>>,
    active_orders: HashMap<Uuid, ActiveOrder>,
}

impl Actor for StockRequester {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Stock requester stopped.");
        for (_, order) in self.active_orders.iter() {
            order.requester.do_send(SendMessage {
                message: ShopResponse::OrderResult {
                    screen_id: order.screen_index,
                    result: Err("Stock requester stopped".to_string()),
                    screen_address: order.screen_address.clone(),
                },
            });
        }
    }
}

impl StockRequester {
    /// Creates a new requester that will forward all stock petitions to given manager.
    pub fn new(stock_manager: Addr<StockManager>) -> Self {
        StockRequester {
            stock_manager,
            connections: HashMap::new(),
            active_orders: HashMap::new(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct MakeOrder {
    pub return_addr: Addr<RobotCommunicator>,
    pub size: u32,
    pub flavors: Vec<String>,
    pub screen_index: usize,
    pub screen_address: String,
}

impl Handler<MakeOrder> for StockRequester {
    type Result = Result<(), String>;

    /// Places an order for the given flavors and the given size with the manager.
    /// The rest of the message fields are identification info used by the rest of the system
    /// to keep track of orders and will be forwarded as needed.
    fn handle(&mut self, msg: MakeOrder, ctx: &mut Self::Context) -> Self::Result {
        let order = ActiveOrder::from_message(&msg);
        debug!("Received order: {:?}", order);
        let id = order.id;
        self.connections.insert(id, msg.return_addr);
        self.active_orders.insert(id, order);

        for flavor in &msg.flavors {
            self.stock_manager
                .try_send(ReserveIceCream {
                    requester_addr: ctx.address().recipient(),
                    request_id: id,
                    flavor: flavor.clone(),
                    amount: msg.size / msg.flavors.len() as u32,
                })
                .map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}

#[derive(Message, PartialEq, Eq, Debug)]
#[rtype(result = "Result<(), String>")]
pub struct StockResult {
    pub requester: Uuid,
    pub result: Option<(String, u32)>,
}

impl Handler<StockResult> for StockRequester {
    type Result = Result<(), String>;

    /// Handles a result given by the stock manager. If an order is finished
    /// or needs to be cancelled, it will notify the robot and remove the order.
    fn handle(&mut self, msg: StockResult, _ctx: &mut Self::Context) -> Self::Result {
        let order = self
            .active_orders
            .get_mut(&msg.requester)
            .ok_or("Order not found")?;
        let mut msg_to_send = None;
        match msg.result {
            Some(flavor) => {
                debug!(
                    "Successfully ordered {} {} for order ID {}",
                    flavor.1,
                    flavor.0,
                    msg.requester.as_fields().0
                );
                order.flavors_ordered.push(flavor.clone());
            }
            None => {
                debug!(
                    "Couldn't get all flavors for order ID {}",
                    msg.requester.as_fields().0
                );
                self.stock_manager
                    .try_send(CancelReserve {
                        reserves: order
                            .flavors_ordered
                            .iter()
                            .map(|(flavor, amount)| (flavor.clone(), *amount))
                            .collect(),
                    })
                    .map_err(|err| err.to_string())?;

                msg_to_send = Some(ShopResponse::OrderResult {
                    screen_id: order.screen_index,
                    result: Err("Couldn't get all flavors".to_string()),
                    screen_address: order.screen_address.clone(),
                })
            }
        };
        if order.is_done() {
            info!("Fulfilled an order.");
            debug!("Order {} finished.", msg.requester.as_fields().0);
            self.stock_manager
                .try_send(ConfirmReserve {
                    reserves: order
                        .flavors_ordered
                        .iter()
                        .map(|(flavor, amount)| (flavor.clone(), *amount))
                        .collect(),
                })
                .map_err(|err| err.to_string())?;
            msg_to_send = Some(ShopResponse::OrderResult {
                screen_id: order.screen_index,
                result: Ok(()),
                screen_address: order.screen_address.clone(),
            });
        }
        if let Some(msg_to_send) = msg_to_send {
            trace!("Sending message to robot: {:?}", msg_to_send);
            let conn = self
                .connections
                .get(&msg.requester)
                .ok_or("Connection not found")?;
            conn.do_send(SendMessage {
                message: msg_to_send,
            });
            self.active_orders.remove(&msg.requester);
        }
        Ok(())
    }
}
