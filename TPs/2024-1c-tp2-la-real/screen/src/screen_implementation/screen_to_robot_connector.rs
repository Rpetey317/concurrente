use actix::{Actor, Addr, AsyncContext};
use std::sync::Arc;
use tokio::{
    io::{split, AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::Mutex,
    task::JoinHandle,
};
use tokio_stream::wrappers::LinesStream;

use super::{
    screen_constants::{CONNECTION_HOST, INITIAL_PORT, LEADER_ELECTION_TIME, MAX_PORT}, screen_order_manager::{AddScreenRobotConnection, ScreenOrderManager, StartSendingOrders}, screen_to_robot_connection::*
};

/// `ScreenConnector` se encarga de crear la conexión a los robots y crear el actor
/// que maneja la conexión y lanzarlo.
pub struct ScreenConnector {}

impl ScreenConnector {
    /// Configura la conexión TCP con el robot líder y crea un actor `ScreenRobotConnection`
    /// para manejar la comunicación.
    ///
    /// # Argumentos
    ///
    /// * `connection_handler_addr`: La dirección (`Addr`) del manejador de órdenes (`ScreenOrderManager`).
    ///
    /// # Retornos
    ///
    /// Devuelve un `JoinHandle` que contiene un `Result` con `()` si la conexión se establece correctamente,
    /// o un `String` en caso de falla.
    ///
    /// Puede devolver un error si no se puede establecer la conexión TCP.
    pub fn handle_connection_with_robot(
        connection_handler_addr: Addr<ScreenOrderManager>,
    ) -> JoinHandle<Result<(), String>> {
        actix::spawn(async move {
            loop {
                for curr_port in INITIAL_PORT..MAX_PORT + 1 {
                    Self::connect_to_robot(curr_port, connection_handler_addr.clone()).await?
                }
                tokio::time::sleep(std::time::Duration::from_millis(LEADER_ELECTION_TIME)).await;
            }
        })
    }

    /// Intenta establecer una conexión TCP con el puerto especificado y crear un actor `ScreenRobotConnection`
    /// para manejar la comunicación.
    ///
    /// # Argumentos
    ///
    /// * `port`: El puerto con el cual intentar la conexión TCP.
    /// * `connection_handler_addr`: La dirección (`Addr`) del manejador de órdenes (`ScreenOrderManager`).
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result` con `()` si la conexión se establece correctamente, o un `String` en caso de falla.
    ///
    /// Puede devolver un error si no se puede establecer la conexión TCP.
    async fn connect_to_robot(
        port: usize,
        connection_handler_addr: Addr<ScreenOrderManager>,
    ) -> Result<(), String> {
        let addr = format!("{}:{}", CONNECTION_HOST, port);
        if let Ok(stream) = TcpStream::connect(addr.clone()).await {
            let (reader, writer) = split(stream);

            let screen_to_robot_connection = ScreenRobotConnection::create(|ctx| {
                ctx.add_stream(LinesStream::new(BufReader::new(reader).lines()));

                ScreenRobotConnection::new(
                    Arc::new(Mutex::new(writer)),
                    connection_handler_addr.clone(),
                )
            });

            connection_handler_addr
                .try_send(AddScreenRobotConnection {
                    addr_screen_robot_connection: screen_to_robot_connection.clone(),
                    port,
                })
                .map_err(|error| error.to_string())?;

            connection_handler_addr
                .try_send(StartSendingOrders {})
                .map_err(|error| error.to_string())?;
            Ok(())
        } else {
            Ok(())
        }
    }
}
