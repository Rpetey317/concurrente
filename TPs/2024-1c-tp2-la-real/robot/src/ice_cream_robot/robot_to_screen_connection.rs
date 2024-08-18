use actix::{
    dev::ContextFutureSpawner, fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context,
    Handler, Message, StreamHandler,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::ice_cream_robot::manage_robot_connection::ReceivedOrderFromScreen;
use common::communication::screen_robot_communication::Request;
use common::communication::shop_requests::ShopRequest;

use super::manage_robot_connection::{
    AskLeaderMessage, ManageRobotConnection, RemoveRobotToScreenConnection,
};

/// `RobotToScreenConnection` es una estructura responsable de gestionar la conexión entre un robot y una pantalla.

pub struct RobotToScreenConnection {
    /// Stream de escritura para la pantalla conectada.
    pub connected_screen_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    /// Dirección del actor `ManageRobotConnection`.
    pub manage_robot_conection: Addr<ManageRobotConnection>,
    /// Dirección del socket de la conexión.
    pub stream_addr: Option<SocketAddr>,
}

impl RobotToScreenConnection {
    /// Crea una nueva instancia de `RobotToScreenConnection`.
    ///
    /// # Argumentos
    ///
    /// * `connected_screen_write_stream` - Stream de escritura para la pantalla conectada.
    /// * `manage_robot_conection` - Dirección del actor `ManageRobotConnection`.
    /// * `stream_addr` - Dirección del socket de la conexión.
    ///
    /// # Retorna
    ///
    /// Una nueva instancia de `RobotToScreenConnection`.
    pub fn new(
        connected_screen_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
        manage_robot_conection: Addr<ManageRobotConnection>,
        stream_addr: SocketAddr,
    ) -> Self {
        Self {
            stream_addr: Some(stream_addr),
            connected_screen_write_stream,
            manage_robot_conection,
        }
    }
}

impl Actor for RobotToScreenConnection {
    type Context = Context<Self>;
}

    /// Maneja mensajes recibidos del stream.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Resultado del mensaje recibido.
    /// * `ctx` - Contexto del actor.
impl StreamHandler<Result<String, std::io::Error>> for RobotToScreenConnection {
    fn handle(&mut self, msg: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(message) = msg {
            if ctx
                .address()
                .try_send(ReceptionOfMessageFromSocketOfScreen {
                    received_message: message,
                })
                .is_err()
            {
                println!("Error receiving message from screen")
            }
        }
    }
/// Maneja la finalización del stream.
    ///
    /// # Argumentos
    ///
    /// * `ctx` - Contexto del actor.
    fn finished(&mut self, ctx: &mut Self::Context) {
        self.manage_robot_conection
            .do_send(RemoveRobotToScreenConnection {
                stream_addr: self.stream_addr.unwrap(),
            });
        ctx.stop();
    }
}


#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para indicar la recepción de un mensaje desde el socket de la pantalla.
struct ReceptionOfMessageFromSocketOfScreen {
    received_message: String,
}


impl Handler<ReceptionOfMessageFromSocketOfScreen> for RobotToScreenConnection {
    type Result = Result<(), String>;

    /// Maneja la recepción de un mensaje desde el socket de la pantalla.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje recibido del socket de la pantalla.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` que contiene `Ok(())` en caso de éxito o un `String` describiendo el error en caso de fallo.
    fn handle(
        &mut self,
        msg: ReceptionOfMessageFromSocketOfScreen,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match serde_json::from_str(&msg.received_message).map_err(|error| error.to_string())? {
            Request::ScreenToRobotOrder {
                index,
                flavors,
                size,
            } => {
                let message = serde_json::to_string(&ShopRequest::IceCreamOrder {
                    flavors,
                    size,
                    screen_id: index,
                    screen_address: self.stream_addr.unwrap().to_string(),
                })
                .map_err(|error| error.to_string())?;
                self.manage_robot_conection
                    .try_send(ReceivedOrderFromScreen { message })
                    .map_err(|error| error.to_string())?;
            }
            Request::ScreenToRobotAskLeader {} => {
                self.manage_robot_conection
                    .try_send(AskLeaderMessage {
                        robot_to_screen_connection_address: ctx.address(),
                    })
                    .map_err(|error| error.to_string())?;
            }
        }

        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para indicar el éxito de una operación para la pantalla.
pub struct SuccessForScreenMessage {
    pub message_to_send_through_socket: String,
}

impl Handler<SuccessForScreenMessage> for RobotToScreenConnection {
    type Result = Result<(), String>;

    /// Maneja el envío de un mensaje de éxito a través del socket de la pantalla.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje de éxito a enviar a través del socket.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` que contiene `Ok(())` en caso de éxito o un `String` describiendo el error en caso de fallo.
    fn handle(&mut self, msg: SuccessForScreenMessage, ctx: &mut Self::Context) -> Self::Result {
        let message = msg.message_to_send_through_socket.clone() + "\n";
        println!("Sucess for screen!, {}", message);
        let connected_screen_write_stream = self.connected_screen_write_stream.clone();
        wrap_future::<_, Self>(async move {
            let _ = connected_screen_write_stream
                .lock()
                .await
                .write_all(message.as_bytes())
                .await
                .is_ok();
        })
        .spawn(ctx);
        Ok(())
    }
}

/// Mensaje para cerrar la conexión desde la pantalla.
#[derive(Message)]
#[rtype(result = "()")]
pub struct CloseConnectionFromScreen;

impl Handler<CloseConnectionFromScreen> for RobotToScreenConnection {
    type Result = ();

    /// Maneja el cierre de la conexión desde la pantalla.
    ///
    /// # Argumentos
    ///
    /// * `_` - Mensaje indicando el cierre de la conexión.
    /// * `ctx` - Contexto del actor.
    fn handle(&mut self, _: CloseConnectionFromScreen, ctx: &mut Self::Context) -> Self::Result {
        if let Some(stream_addr) = self.stream_addr {
            self.manage_robot_conection
                .do_send(RemoveRobotToScreenConnection { stream_addr });
        }

        ctx.stop();
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para enviar datos a través del socket a la pantalla.
pub struct SendMessageThroughSocketToScreen {
    pub message_to_send_through_socket: String,
}

impl Handler<SendMessageThroughSocketToScreen> for RobotToScreenConnection {
    type Result = Result<(), String>;

    /// Maneja el envío de un mensaje a través del socket a la pantalla.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje a enviar a través del socket.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` que contiene `Ok(())` en caso de éxito o un `String` describiendo el error en caso de fallo.
    fn handle(
        &mut self,
        msg: SendMessageThroughSocketToScreen,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let online_msg = msg.message_to_send_through_socket.clone() + "\n";
        let screen_write_stream = self.connected_screen_write_stream.clone();
        wrap_future::<_, Self>(async move {
            let _ = screen_write_stream
                .lock()
                .await
                .write_all(online_msg.as_bytes())
                .await
                .is_ok();
        })
        .spawn(ctx);
        Ok(())
    }
}
