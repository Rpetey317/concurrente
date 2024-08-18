use crate::screen_implementation::screen_order_manager::*;
use crate::screen_implementation::screen_order_parser::ScreenOrderParser;
use crate::screen_implementation::screen_to_robot_connector::ScreenConnector;
use crate::screen_implementation::user_input::UserInput;
use actix::{Actor, Addr};
use actix_rt::System;

use super::screen_constants::RELATIVE_PATH;

/// Función principal de arranque. Se encarga de iniciar en orden las cosas necesarias para poder procesar las órdenes.
///
/// # Argumentos
///
/// * `orders_path` - Ruta relativa al archivo CSV que contiene las órdenes a procesar.
///
/// # Errores
///
/// Retorna un `Result<(), String>` indicando éxito o error durante el proceso de inicio.
///
pub fn start(orders_path: String) -> Result<(), String> {
    let relative_orders_path = RELATIVE_PATH.to_owned() + &orders_path;

    if !std::path::Path::new(&relative_orders_path).exists() {
        return Err(
            "Relative orders path does not exist. Realtive path starts from root execution.".into(),
        );
    }

    let orders_parser = ScreenOrderParser::new(relative_orders_path);

    let local_orders = orders_parser.get_orders()?;

    let user_input_handler = UserInput::start_user_input();
    let system = System::new();

    system.block_on(start_async_orders(local_orders))?;

    system.run().map_err(|_| "Error while running")?;
    user_input_handler
        .join()
        .map_err(|_| "Error while joining user input handler thread")??;

    Ok(())
}

/// Función asíncrona que se encarga de lanzar los actores y enviar las órdenes. No termina hasta que el usuario no cierre la entrada de usuario.
///
/// # Argumentos
///
/// * `local_orders` - Vectores de cadenas que representan las órdenes a procesar.
///
/// # Errores
///
/// Retorna un `Result<(), String>` indicando éxito o error durante el proceso de inicio asíncrono.
///
async fn start_async_orders(local_orders: Vec<Vec<String>>) -> Result<(), String> {
    let connection_handler = start_connection_handler(local_orders)?;

    ScreenConnector::handle_connection_with_robot(connection_handler);

    Ok(())
}

/// Función que inicializa el manejador de conexiones, que en este caso es un actor de Actix que se encarga de procesar las órdenes.
///
/// # Argumentos
///
/// * `orders` - Vectores de cadenas que representan las órdenes a procesar.
///
/// # Errores
///
/// Retorna un `Result<Addr<ScreenOrderManager>, String>` indicando éxito o error durante el proceso de inicio del manejador de conexiones.
///
fn start_connection_handler(orders: Vec<Vec<String>>) -> Result<Addr<ScreenOrderManager>, String> {
    let connection_handler = ScreenOrderManager::new_manager_with_orders(orders).start();

    Ok(connection_handler)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_with_invalid_orders_path() {
        let orders_path = "invalid_path.csv".to_string();
        let result = start(orders_path);
        assert!(result.is_err());
    }
}
