use std::sync::{mpsc, Arc};

use super::stock_requester::StockRequester;
use crate::shop_connection::robot_communicator::RobotCommunicator;
use crate::shop_stock::stock_manager::{BackupShop, StockManager};
use crate::utils::command_interpreter::{BACKUP, EXIT};
use actix::prelude::*;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    sync::Mutex,
    task::JoinHandle,
};
use tokio_stream::wrappers::LinesStream;
use tracing::info;

/// Starts a new async task for listening to incoming connections in the given port.
/// Any new connection is automatically handled by the responsible actors.
pub fn handle_incoming_connections(
    port: &str,
    stock_file: &str,
    backup_prefix: String,
    interpreter_listener_recv: mpsc::Receiver<String>,
    max_orders_before_backup: usize,
) -> JoinHandle<()> {
    let addr = format!("127.0.0.1:{}", port);

    let stock_manager = StockManager::new(stock_file, max_orders_before_backup).start();

    actix::spawn(async move {
        let _ = accept_connections(
            addr,
            &stock_manager,
            &backup_prefix,
            interpreter_listener_recv,
        )
        .await;
    })
}

/// Main loop of accepting connections and creating necessary actors
///
/// Will run until a signal from the command interpreter is received
async fn accept_connections(
    addr: String,
    stock: &Addr<StockManager>,
    backup_prefix: &str,
    interpreter_listener_recv: mpsc::Receiver<String>,
) -> Result<(), String> {
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|err| err.to_string())?;
    info!("Listening for connections on: {}", addr);

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            if let Some(cmd) = execute_input(&interpreter_listener_recv, stock, backup_prefix) {
                if cmd == EXIT {
                    return Ok(());
                }
                continue;
            }
            let (reader, writer) = tokio::io::split(stream);
            let writer = Arc::new(Mutex::new(writer));
            let requester = StockRequester::new(stock.clone()).start();
            RobotCommunicator::create(|ctx| {
                RobotCommunicator::add_stream(
                    LinesStream::new(BufReader::new(reader).lines()),
                    ctx,
                );
                info!("Established new connection with robot.");
                RobotCommunicator::new(writer.clone(), requester.clone())
            });
        }
    }
}

fn execute_input(
    interpreter_listener_recv: &mpsc::Receiver<String>,
    stock: &Addr<StockManager>,
    backup_prefix: &str,
) -> Option<String> {
    if let Ok(msg) = interpreter_listener_recv.try_recv() {
        match msg.as_str() {
            EXIT => {
                info!("Received exit command. Stopping system");
                return Some(EXIT.to_string());
            }
            BACKUP => {
                info!("Received backup command. Creating backup");
                let out_dir = "backups".to_string();
                let out_file = format!(
                    "{}-{}.csv",
                    backup_prefix,
                    chrono::Utc::now().timestamp()
                );
                stock.do_send(BackupShop { 
                    out_dir,
                    out_file,
                });
                return Some(BACKUP.to_string());
            }
            _ => return None,
        }
    }
    None
}
