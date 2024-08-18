use std::error::Error;

use actix::prelude::*;
use actix::Addr;

use tokio::join;

use super::robot_to_robot_connector::RobotToRobotConnector;
use super::{
    ice_cream_shop_connector::IceCreamShopConnector,
    manage_robot_connection::{AddIceCreamShopConnectionMessage, ManageRobotConnection},
    robot_to_screen_connector::RobotToScreenConnector,
    user_input::UserInput,
};

/// Función para iniciar el servicio.
///
/// # Argumentos
///
/// * `robot_to_robot_port` - Puerto para la conexión entre robots.
/// * `screens_port` - Puerto para la conexión con las pantallas.
///
/// # Retorna
///
/// Un `Result` que es `Ok` si el servicio se inició correctamente, o un `Box<dyn Error>` en caso de error.
pub fn start_serving(
    robot_to_robot_port: usize,
    screens_port: usize,
) -> Result<(), Box<dyn Error>> {
    let user_input = UserInput::start_user_input();

    System::new().block_on(start_serving_async(robot_to_robot_port, screens_port))?;

    user_input.join().map_err(|_| "Error in user_input")??;

    Ok(())
}

/// Función asincrónica para iniciar el servicio.
///
/// # Argumentos
///
/// * `robot_to_robot_port` - Puerto para la conexión entre robots.
/// * `screens_port` - Puerto para la conexión con las pantallas.
///
/// # Retorna
///
/// Un `Result` que es `Ok` si el servicio se inició correctamente, o un `Box<dyn Error>` en caso de error.
async fn start_serving_async(
    robot_to_robot_port: usize,
    screens_port: usize,
) -> Result<(), Box<dyn Error>> {
    let manage_robot_conection = start_actors(robot_to_robot_port, screens_port).await?;

    let robot_to_screen_connection =
        RobotToScreenConnector::setup_connection(manage_robot_conection.clone(), screens_port);

    let robot_to_robot_connection = RobotToRobotConnector::setup_connection(
        manage_robot_conection.clone(),
        robot_to_robot_port,
    );

    let (join_result_screen, join_result_robot) =
        join!(robot_to_screen_connection, robot_to_robot_connection);

    join_result_screen.map_err(|error| error.to_string())??;
    join_result_robot.map_err(|error| error.to_string())??;

    Ok(())
}

/// Función asincrónica para iniciar los actores.
///
/// # Argumentos
///
/// * `robot_to_robot_id` - ID del robot para la conexión entre robots.
/// * `robot_to_screen_id` - ID del robot para la conexión con las pantallas.
///
/// # Retorna
///
/// Un `Result` con la dirección de `ManageRobotConnection` si los actores se iniciaron correctamente, o un `Box<dyn Error>` en caso de error.
async fn start_actors(
    robot_to_robot_id: usize,
    robot_to_screen_id: usize,
) -> Result<Addr<ManageRobotConnection>, Box<dyn Error>> {
    let manage_robot_conection =
        ManageRobotConnection::new(robot_to_robot_id, robot_to_screen_id).start();

    let robot_to_ice_cream_shop_connection =
        IceCreamShopConnector::setup_connection(manage_robot_conection.clone()).await?;

    manage_robot_conection
        .send(AddIceCreamShopConnectionMessage {
            robot_to_ice_cream_shop_connection: robot_to_ice_cream_shop_connection.clone(),
        })
        .await??;

    Ok(manage_robot_conection)
}
