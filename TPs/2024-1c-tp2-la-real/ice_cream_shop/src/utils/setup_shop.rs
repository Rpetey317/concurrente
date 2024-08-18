use actix::prelude::*;
use std::sync::mpsc::{self, channel};
use tracing::{debug, info};

use crate::{
    shop_connection,
    utils::{arg_parsing::Args, command_interpreter},
};

/// Main function of the program. Sets up the shop actors and starts the command interpreter thread.
pub fn setup_shop(args: Args) -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_max_level(args.tracing_level)
        .init();
    info!("Setting up everything...");
    debug!("Arguments received: {:?}", args);

    let (setup_interpreter_send, setup_interpreter_recv) = channel::<mpsc::Sender<String>>();

    let interpreter_handle =
        command_interpreter::handle_commands(setup_interpreter_recv, args.port.clone());

    System::new().block_on(run_shop(
        setup_interpreter_send,
        args.port,
        args.input_file,
        args.backup_file_prefix,
        args.max_orders_before_backup,
    ))?;

    interpreter_handle
        .join()
        .map_err(|_| "Error joining cmd interpreter handle")??;
    Ok(())
}

/// Async main of the program.
async fn run_shop(
    setup_interpreter_send: mpsc::Sender<mpsc::Sender<String>>,
    port: String,
    input_file: String,
    backup_prefix: String,
    max_orders_before_backup: usize,
) -> Result<(), String> {
    let (interpreter_listener_send, interpreter_listener_recv) = channel::<String>();

    let shop_handle = shop_connection::robot_listener::handle_incoming_connections(
        &port,
        &input_file,
        backup_prefix,
        interpreter_listener_recv,
        max_orders_before_backup,
    );

    setup_interpreter_send
        .send(interpreter_listener_send)
        .map_err(|_| "Error sending channel to interpreter")?;

    shop_handle
        .await
        .map_err(|_| "Error joining listener handle")?;

    Ok(())
}
