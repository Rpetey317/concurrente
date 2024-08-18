use std::sync::Arc;

use actix::{
    dev::ContextFutureSpawner, fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler,
    Message, StreamHandler,
};
use common::communication::screen_robot_communication::*;
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use super::screen_order_manager::*;

/// `ScreenRobotConnection` se ocupa de la conexión con los robots, encapsulando el socket
/// y permitiendo recibir y enviar información.
pub struct ScreenRobotConnection {
    connected_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    order_manager: Addr<ScreenOrderManager>,
}

impl Actor for ScreenRobotConnection {
    type Context = Context<Self>;
}

impl ScreenRobotConnection {
    /// Crea una nueva instancia de `ScreenRobotConnection`.
    ///
    /// # Argumentos
    ///
    /// * `connected_write_stream`: Instancia de `WriteHalf<TcpStream>` encapsulada en `Arc<Mutex>`.
    /// * `order_manager`: Dirección (`Addr`) del manejador de órdenes (`ScreenOrderManager`).
    ///
    /// # Retorno
    ///
    /// Retorna una nueva instancia de `ScreenRobotConnection`.
    pub fn new(
        connected_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
        order_manager: Addr<ScreenOrderManager>,
    ) -> Self {
        Self {
            connected_write_stream,
            order_manager,
        }
    }
}

/// Mensaje para añadir el manager de órdenes (`ScreenOrderManager`) a la conexión.
/// # Atributos del msg
///
/// * `address`: Dirección de un `ScreenOrderManager`, que será luego usada para comunicación con el actor.
#[derive(Message, Debug, PartialEq, Eq)]
#[rtype(result = "Result<(), String>")]
pub struct AddScreenOrderManager {
    pub address: Addr<ScreenOrderManager>,
}

impl Handler<AddScreenOrderManager> for ScreenRobotConnection {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: AddScreenOrderManager, _ctx: &mut Self::Context) -> Self::Result {
        self.order_manager = msg.address.clone();
        Ok(())
    }
}

/// Mensaje para manejar la respuesta recibida de los robots y reenviarla al `OrderManager` para cerrar las órdenes.
/// # Atributos del msg
///
/// * `result_for_order_handler`: El resultado recibido que queremos enviar al `OrderManager`.
#[derive(Message, Debug, PartialEq, Eq)]
#[rtype(result = "Result<(), String>")]
pub struct HandleRecievedMessage {
    pub result_for_order_handler: String,
}

impl Handler<HandleRecievedMessage> for ScreenRobotConnection {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: HandleRecievedMessage, ctx: &mut Self::Context) -> Self::Result {
        match serde_json::from_str(&msg.result_for_order_handler)
            .map_err(|error| error.to_string())?
        {
            Response::RobotToScreenResult { index, result } => {
                self.order_manager
                    .try_send(ReturnedOrderFromShop {
                        result,
                        index: index as u32,
                    })
                    .map_err(|error| error.to_string())?;
                Ok(())
            }
            Response::RobotToScreenLeaderPort { leader_port } => ctx
                .address()
                .try_send(ReceivedLeaderMessage { leader_port })
                .map_err(|error| error.to_string()),
        }
    }
}

/// Mensaje para manejar la recepción del mensaje del líder.
/// # Atributos del msg
///
/// * `leader_port`: El puerto del líder recibido.
#[derive(Message, Debug)]
#[rtype(result = "Result<(), String>")]
pub struct ReceivedLeaderMessage {
    pub leader_port: usize,
}

impl Handler<ReceivedLeaderMessage> for ScreenRobotConnection {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: ReceivedLeaderMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.order_manager
            .try_send(LeaderMessage {
                leader_server_id: msg.leader_port,
            })
            .map_err(|error| error.to_string())
    }
}

/// Handler de mensajes de entrada. Envía los mensajes recibidos al `OrderManager` para su procesamiento.
impl StreamHandler<Result<String, std::io::Error>> for ScreenRobotConnection {
    fn handle(&mut self, msg: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(message) = msg {
            let _ = ctx
                .address()
                .try_send(HandleRecievedMessage {
                    result_for_order_handler: message,
                })
                .map_err(|error| error.to_string());
        }
    }
}

/// Mensaje para enviar un mensaje al robot líder a través del socket.
/// # Atributos del msg
///
/// * `message_to_send_through_socket`: String que es el mensaje que queremos enviar.
#[derive(Message, Debug, PartialEq, Eq)]
#[rtype(result = "Result<(), String>")]
pub struct SendMessageThroughSocket {
    pub message_to_send_through_socket: String,
}

impl Handler<SendMessageThroughSocket> for ScreenRobotConnection {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: SendMessageThroughSocket, ctx: &mut Self::Context) -> Self::Result {
        let message = msg.message_to_send_through_socket.clone() + "\n";
        let writer = self.connected_write_stream.clone();
        wrap_future::<_, Self>(async move {
            let _ = writer
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
