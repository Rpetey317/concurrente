use std::sync::Arc;

use actix::{
    dev::ContextFutureSpawner, fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context,
    Handler, Message, StreamHandler,
};
use common::communication::robot_to_robot_communication::RobotRequest;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::ice_cream_robot::manage_robot_connection::RegisterRobotToRobotConnectionMessage;

use super::manage_robot_connection::{
    ElectionForDeadRobot, StartLeaderElection, LeaderSelected, ManageRobotConnection,
};

/// `RobotToRobotConnection` es una estructura que representa la conexión entre robots.
pub struct RobotToRobotConnection {
    pub manage_robot_connection: Addr<ManageRobotConnection>,
    pub connected_robot_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub connected_robot_to_robot_id: Option<usize>,
}

impl RobotToRobotConnection {

    /// Crea una nueva instancia de `RobotToRobotConnection`.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_connection` - Dirección del actor `ManageRobotConnection`.
    /// * `connected_robot_write_stream` - Stream de escritura del robot conectado.
    ///
    /// # Retorna
    ///
    /// Una nueva instancia de `RobotToRobotConnection`.
    pub fn new(
        manage_robot_connection: Addr<ManageRobotConnection>,
        connected_robot_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    ) -> Self {
        RobotToRobotConnection {
            manage_robot_connection,
            connected_robot_write_stream,
            connected_robot_to_robot_id: None,
        }
    }
}

impl Actor for RobotToRobotConnection {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for RobotToRobotConnection {

    /// Maneja los mensajes recibidos desde otro robot.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje recibido.
    /// * `ctx` - Contexto del actor.
    fn handle(&mut self, msg: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(msg) = msg {
            if ctx
                .address()
                .try_send(ReceptionOfMessageFromSocketOfRobot {
                    received_message: msg,
                })
                .is_err()
            {
                println!("Error receiving message from another robot")
            }
        }
    }

    /// Se llama cuando el `StreamHandler` termina.
    ///
    /// # Argumentos
    ///
    /// * `ctx` - Contexto del actor.
    fn finished(&mut self, ctx: &mut Self::Context) {
        if let Some(server_id) = self.connected_robot_to_robot_id {
            println!(
                "A robot with id <{}> died! we need an election. I love democracy",
                server_id
            );
            self.manage_robot_connection.do_send(ElectionForDeadRobot {
                closed_server_id: server_id,
            });
        }
        ctx.stop();
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para indicar la recepción de un mensaje desde el socket de otro robot.
struct ReceptionOfMessageFromSocketOfRobot {
    received_message: String,
}

impl Handler<ReceptionOfMessageFromSocketOfRobot> for RobotToRobotConnection {
    type Result = Result<(), String>;

    /// Maneja el mensaje de recepción de un mensaje desde el socket de otro robot.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje recibido.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` indicando el éxito o fracaso de la operación.
    fn handle(
        &mut self,
        msg: ReceptionOfMessageFromSocketOfRobot,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match serde_json::from_str(&msg.received_message).map_err(|error| error.to_string())? {
            RobotRequest::GetMyInformation { robot_to_robot_id } => {
                self.connected_robot_to_robot_id = Some(robot_to_robot_id);
                self.manage_robot_connection
                    .try_send(RegisterRobotToRobotConnectionMessage {
                        robot_to_robot_id,
                        robot_to_robot_connection: ctx.address(),
                    })
                    .map_err(|error| error.to_string())?;
            }
            RobotRequest::StartElection {} => {
                self.manage_robot_connection
                    .try_send(StartLeaderElection {})
                    .map_err(|error| error.to_string())?;
            }
            RobotRequest::LeaderSelected {
                robot_to_robot_leader_id,
                robot_to_screen_leader_id,
            } => {
                self.manage_robot_connection
                    .try_send(LeaderSelected {
                        robot_to_robot_leader_id,
                        robot_to_screen_leader_id,
                    })
                    .map_err(|error| error.to_string())?;
            }
        }

        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para enviar datos a través del socket a otro robot.
pub struct SendMessageThroughSocketToRobot {
    pub message_to_send_through_socket: String,
}

impl Handler<SendMessageThroughSocketToRobot> for RobotToRobotConnection {
    type Result = Result<(), String>;

    /// Maneja el mensaje para enviar datos a través del socket a otro robot.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje con los datos a enviar.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` indicando el éxito o fracaso de la operación.
    fn handle(
        &mut self,
        msg: SendMessageThroughSocketToRobot,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let online_msg = msg.message_to_send_through_socket.clone() + "\n";
        let robot_write_stream = self.connected_robot_write_stream.clone();
        wrap_future::<_, Self>(async move {
            if robot_write_stream
                .lock()
                .await
                .write_all(online_msg.as_bytes())
                .await
                .is_ok()
            {}
        })
        .spawn(ctx);
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(),String>")]
/// Mensaje para iniciar elecciones.
pub struct SendStartElections {}

impl Handler<SendStartElections> for RobotToRobotConnection {
    type Result = Result<(), String>;

    /// Maneja el mensaje para iniciar elecciones.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje para iniciar elecciones.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` indicando el éxito o fracaso de la operación.
    fn handle(&mut self, _msg: SendStartElections, ctx: &mut Self::Context) -> Self::Result {
        let message = serde_json::to_string(&RobotRequest::StartElection {})
            .map_err(|error| error.to_string())?;

        ctx.address()
            .try_send(SendMessageThroughSocketToRobot {
                message_to_send_through_socket: message,
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(),String>")]
/// `SendLeaderSelected` es un mensaje que contiene la información sobre el líder seleccionado.
pub struct SendLeaderSelected {
    pub robot_to_robot_id: usize,
    pub robot_to_screen_id: usize,
}

impl Handler<SendLeaderSelected> for RobotToRobotConnection {
    type Result = Result<(), String>;

    /// Maneja el mensaje `SendLeaderSelected`.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `SendLeaderSelected` que contiene los IDs del líder seleccionado.
    /// * `ctx` - El contexto del actor `RobotToRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación fue exitosa o no.
    fn handle(&mut self, msg: SendLeaderSelected, ctx: &mut Self::Context) -> Self::Result {
        let message = serde_json::to_string(&RobotRequest::LeaderSelected {
            robot_to_robot_leader_id: msg.robot_to_robot_id,
            robot_to_screen_leader_id: msg.robot_to_screen_id,
        })
        .map_err(|error| error.to_string())?;

        ctx.address()
            .try_send(SendMessageThroughSocketToRobot {
                message_to_send_through_socket: message,
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}
