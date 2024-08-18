use std::sync::Arc;

use actix::{Actor, Addr, AsyncContext};
use tokio::{
    io::{split, AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};
use tokio_stream::wrappers::LinesStream;

use super::{
    ice_cream_shop_connection::IceCreamShopConnection,
    manage_robot_connection::ManageRobotConnection,
    robot_constants::{CONNECTION_HOST, ICE_CREAM_SHOP_PORT},
};

pub struct IceCreamShopConnector {}

impl IceCreamShopConnector {
    /// Configura la conexión con la tienda de helados.
    ///
    /// # Argumentos
    ///
    /// * `manage_robot_conection` - Dirección del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` con la dirección del actor `IceCreamShopConnection` si la conexión se configuró correctamente, o un `String` en caso de error.
    pub async fn setup_connection(
        manage_robot_conection: Addr<ManageRobotConnection>,
    ) -> Result<Addr<IceCreamShopConnection>, String> {
        let address = format!("{}:{}", CONNECTION_HOST, ICE_CREAM_SHOP_PORT);

        let stream = TcpStream::connect(address.clone())
            .await
            .map_err(|error| error.to_string())?;

        let (reader, writer) = split(stream);

        let ice_cream_shop_connection = IceCreamShopConnection::create(|ctx| {
            ctx.add_stream(LinesStream::new(BufReader::new(reader).lines()));
            IceCreamShopConnection::new(manage_robot_conection, Arc::new(Mutex::new(writer)))
        });

        Ok(ice_cream_shop_connection)
    }
}
