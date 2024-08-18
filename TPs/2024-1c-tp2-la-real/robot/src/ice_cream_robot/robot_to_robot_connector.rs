use std::sync::Arc;

use actix::{Actor, Addr, AsyncContext};
use tokio::{
    io::{split, AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
};
use tokio_stream::wrappers::LinesStream;

use super::{
    manage_robot_connection::{
        AddRobotToRobotConnectionMessage, StartLeaderElection, ManageRobotConnection,
    },
    robot_constants::{CONNECTION_HOST, INITIAL_CURRENT_PORT, MAX_PORT},
    robot_to_robot_connection::RobotToRobotConnection,
};

/// `RobotToRobotConnector` es una estructura responsable de gestionar las conexiones entre robots.
pub struct RobotToRobotConnector {}

impl RobotToRobotConnector {
    /// Configura una conexión entre robots.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_connection` - Dirección del actor `ManageRobotConnection`.
    /// * `robot_to_robot_port` - Puerto para la conexión entre robots.
    ///
    /// # Retorna
    ///
    /// Un `JoinHandle` que contiene `Result<(), String>` indicando el resultado de la operación.
    pub fn setup_connection(
        manage_robot_connection: Addr<ManageRobotConnection>,
        robot_to_robot_port: usize,
    ) -> JoinHandle<Result<(), String>> {
        actix::spawn(async move {
            Self::manage_connection(manage_robot_connection, robot_to_robot_port).await
        })
    }

    /// Gestiona la conexión entre robots.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_connection` - Dirección del actor `ManageRobotConnection`.
    /// * `robot_to_robot_port` - Puerto para la conexión entre robots.
    ///
    /// # Retorna
    ///
    /// Un `Result<(), String>` indicando el resultado de la operación.
    async fn manage_connection(
        manage_robot_connection: Addr<ManageRobotConnection>,
        robot_to_robot_port: usize,
    ) -> Result<(), String> {
        Self::try_connection(manage_robot_connection.clone()).await?;

        manage_robot_connection
            .try_send(StartLeaderElection {})
            .map_err(|error| error.to_string())?;

        let listener = TcpListener::bind(format!("{}:{}", CONNECTION_HOST, robot_to_robot_port))
            .await
            .map_err(|error| error.to_string())?;

        loop {
            if let Ok((stream, _)) = listener.accept().await {
                Self::manage_connected_robot(stream, &manage_robot_connection)?;
            };
        }
    }

    /// Intenta establecer una conexión entre robots.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_connection` - Dirección del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result<(), String>` indicando el resultado de la operación.
    async fn try_connection(
        manage_robot_connection: Addr<ManageRobotConnection>,
    ) -> Result<(), String> {
        let mut current_port = INITIAL_CURRENT_PORT;
        while current_port <= MAX_PORT {
            let address = format!("{}:{}", CONNECTION_HOST, current_port);

            if let Ok(stream) = TcpStream::connect(address.clone()).await {
                let (reader, writer) = split(stream);
                let robot_to_robot_connection = RobotToRobotConnection::create(|ctx| {
                    ctx.add_stream(LinesStream::new(BufReader::new(reader).lines()));
                    RobotToRobotConnection::new(
                        manage_robot_connection.clone(),
                        Arc::new(Mutex::new(writer)),
                    )
                });
                manage_robot_connection
                    .try_send(AddRobotToRobotConnectionMessage {
                        robot_to_robot_id: Some(current_port),
                        robot_to_robot_connection_address: robot_to_robot_connection,
                    })
                    .map_err(|error| error.to_string())?;
            }
            current_port += 1;
        }

        Ok(())
    }

    /// Gestiona un robot conectado.
    ///
    /// # Argumentos
    ///
    /// * `async_stream` - Stream asincrónico del robot conectado.
    /// * `manage_robot_connection` - Dirección del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result<(), String>` indicando el resultado de la operación.
    fn manage_connected_robot(
        async_stream: TcpStream,
        manage_robot_connection: &Addr<ManageRobotConnection>,
    ) -> Result<(), String> {
        let (reader, writer) = split(async_stream);

        let robot_to_robot_connection = RobotToRobotConnection::create(|ctx| {
            ctx.add_stream(LinesStream::new(BufReader::new(reader).lines()));
            RobotToRobotConnection::new(
                manage_robot_connection.clone(),
                Arc::new(Mutex::new(writer)),
            )
        });

        manage_robot_connection
            .try_send(AddRobotToRobotConnectionMessage {
                robot_to_robot_id: None,
                robot_to_robot_connection_address: robot_to_robot_connection,
            })
            .map_err(|error| error.to_string())
    }
}
