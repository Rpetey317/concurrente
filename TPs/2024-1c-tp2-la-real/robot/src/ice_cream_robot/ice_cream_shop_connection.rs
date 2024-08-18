use std::sync::Arc;

use actix::{
    dev::ContextFutureSpawner, fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler,
    Message, StreamHandler,
};
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use super::manage_robot_connection::{ManageRobotConnection, SolvedOrderFromIceCreamShop};

/// Estructura para gestionar la conexión con la tienda de helados.
pub struct IceCreamShopConnection {
    manage_robot_conection: Addr<ManageRobotConnection>,
    ice_cream_shop_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
}

impl Actor for IceCreamShopConnection {
    type Context = Context<Self>;
}

impl IceCreamShopConnection {
    /// Crea una nueva instancia de `IceCreamShopConnection`.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_conection` - Dirección del actor `ManageRobotConnection`.
    /// * `ice_cream_shop_write_stream` - Stream de escritura para la tienda de helados.
    ///
    /// # Retorna
    ///
    /// Una nueva instancia de `IceCreamShopConnection`.
    pub fn new(
        manage_robot_conection: Addr<ManageRobotConnection>,
        ice_cream_shop_write_stream: Arc<Mutex<WriteHalf<TcpStream>>>,
    ) -> Self {
        Self {
            manage_robot_conection,
            ice_cream_shop_write_stream,
        }
    }
}

impl StreamHandler<Result<String, std::io::Error>> for IceCreamShopConnection {
    /// Maneja los mensajes recibidos desde la tienda de helados.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje recibido.
    /// * `_ctx` - Contexto del actor.
    fn handle(&mut self, msg: Result<String, std::io::Error>, _ctx: &mut Self::Context) {
        if let Ok(message) = msg {
            if self
                .manage_robot_conection
                .try_send(SolvedOrderFromIceCreamShop { message })
                .is_err()
            {
                println!("Error receiving message from ice cream shop")
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para enviar datos a través del socket a la tienda de helados.
pub struct SendMessageThroughSocketToIceCreamShop {
    pub message_to_send_through_socket: String,
}

impl Handler<SendMessageThroughSocketToIceCreamShop> for IceCreamShopConnection {
    type Result = Result<(), String>;

    /// Maneja el mensaje para enviar datos a través del socket a la tienda de helados.
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
        msg: SendMessageThroughSocketToIceCreamShop,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let message = msg.message_to_send_through_socket.clone() + "\n";
        let ice_cream_shop_write_stream = self.ice_cream_shop_write_stream.clone();
        wrap_future::<_, Self>(async move {
            let _ = ice_cream_shop_write_stream
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

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para solicitar un sabor de helado.
pub struct RequestIceCreamFlavor {
    pub message: String,
}

impl Handler<RequestIceCreamFlavor> for IceCreamShopConnection {
    type Result = Result<(), String>;
    
    /// Maneja el mensaje para solicitar un sabor de helado.
    ///
    /// # Argumentos
    ///
    /// * `msg` - Mensaje con la solicitud del sabor.
    /// * `ctx` - Contexto del actor.
    ///
    /// # Retorna
    ///
    /// Un `Result` indicando el éxito o fracaso de la operación.
    fn handle(&mut self, msg: RequestIceCreamFlavor, ctx: &mut Self::Context) -> Self::Result {
        ctx.address()
            .try_send(SendMessageThroughSocketToIceCreamShop {
                message_to_send_through_socket: msg.message,
            })
            .map_err(|error| error.to_string())
    }
}
