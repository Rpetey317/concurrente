use actix::{Actor, Addr, AsyncContext};
use tokio::{
    io::{split, AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
};
use tokio_stream::wrappers::LinesStream;

use crate::ice_cream_robot::manage_robot_connection::AddRobotToScreenConnectionMessage;

use super::{
    manage_robot_connection::ManageRobotConnection, robot_constants::CONNECTION_HOST,
    robot_to_screen_connection::RobotToScreenConnection,
};

use std::{net::SocketAddr, sync::Arc};

/// `RobotToScreenConnector` es una estructura responsable de establecer y gestionar conexiones entre robots y pantallas.
pub struct RobotToScreenConnector {}

impl RobotToScreenConnector {
    /// Configura la conexión inicial para el manejo de las conexiones de robots a pantallas.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_conection` - Dirección del actor `ManageRobotConnection`.
    /// * `screens_port` - Puerto donde se escucharán las conexiones de las pantallas.
    ///
    ///
    /// # Retorna
    ///
    /// Un `Result` que contiene `Ok(())` en caso de éxito o un `String` describiendo el error en caso de fallo.
    pub fn setup_connection(
        manage_robot_conection: Addr<ManageRobotConnection>,
        screens_port: usize,
    ) -> JoinHandle<Result<(), String>> {
        actix::spawn(
            async move { Self::manage_connection(manage_robot_conection, screens_port).await },
        )
    }

    /// Gestiona todas las conexion recibidas de las pantallas
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_conection` - Dirección del actor `ManageRobotConnection`.
    /// * `screens_port` - Puerto donde se escucharán las conexiones de las pantallas.
    ///
    /// # Retorna
    ///
    /// Un `Result` que contiene `Ok(())` en caso de éxito o un `String` describiendo el error en caso de fallo.
    async fn manage_connection(
        manage_robot_conection: Addr<ManageRobotConnection>,
        screens_port: usize,
    ) -> Result<(), String> {
        loop {
            let listener = TcpListener::bind(format!("{}:{}", CONNECTION_HOST, screens_port))
                .await
                .map_err(|error| error.to_string())?;

            match listener.accept().await {
                Ok((stream, stream_addr)) => {
                    Self::manage_connected_screen(stream, stream_addr, &manage_robot_conection)?;
                }
                Err(error) => {
                    return Err(error.to_string());
                }
            };
        }
    }

    /// Gestiona la conexión de una pantalla ya conectada y la añade al manage_robot_connection.
    ///
    /// # Argumentos
    ///
    /// * `stream` - Stream de la conexión TCP de la pantalla.
    /// * `stream_addr` - Dirección del socket de la pantalla.
    /// * `manage_robot_conection` - Referencia a la dirección del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que contiene `Ok(())` en caso de éxito o un `String` describiendo el error en caso de fallo.
    fn manage_connected_screen(
        stream: TcpStream,
        stream_addr: SocketAddr,
        manage_robot_conection: &Addr<ManageRobotConnection>,
    ) -> Result<(), String> {
        let (read_half, write_half) = split(stream);

        RobotToScreenConnection::create(|ctx| {
            manage_robot_conection.do_send(AddRobotToScreenConnectionMessage {
                address: stream_addr,
                robot_to_screen_connection: ctx.address(),
            });

            ctx.add_stream(LinesStream::new(BufReader::new(read_half).lines()));
            RobotToScreenConnection::new(
                Arc::new(Mutex::new(write_half)),
                manage_robot_conection.clone(),
                stream_addr,
            )
        });

        Ok(())
    }
}
