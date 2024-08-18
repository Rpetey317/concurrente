use std::{net::TcpStream, sync::mpsc};

use actix::prelude::*;
use std::thread::JoinHandle;
use tracing::{info, warn};

pub const EXIT: &str = "q";
pub const BACKUP: &str = "b";

/// Thread in charge of receiving inputs from stdin while the program is running, and informing
/// the rest of the system of the commands received.
pub fn handle_commands(
    setup_interpreter_recv: mpsc::Receiver<mpsc::Sender<String>>,
    shop_port: String,
) -> JoinHandle<Result<(), String>> {
    std::thread::spawn(move || -> Result<(), String> {
        info!("Command interpreter started");
        let mut reader = std::io::stdin().lines();

        let interpreter_listener_send = setup_interpreter_recv
            .recv()
            .map_err(|err| err.to_string())?;

        while let Some(Ok(line)) = reader.next() {
            match line.as_str() {
                EXIT => {
                    info!("Interpreter received exit command");
                    let _ = interpreter_listener_send.send(EXIT.to_string());
                    let _ = TcpStream::connect(format!("127.0.0.1:{}", shop_port));

                    if let Some(system) = System::try_current() {
                        info!("Stopping system");
                        system.stop()
                    }
                    break;
                }
                BACKUP => {
                    info!("Interpreter received backup command");
                    let _ = interpreter_listener_send.send(BACKUP.to_string());
                    let _ = TcpStream::connect(format!("127.0.0.1:{}", shop_port));
                }
                _ => {
                    warn!("Unknown command. Available commands: {}, {}.", EXIT, BACKUP);
                }
            }
        }
        Ok(())
    })
}
