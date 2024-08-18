pub mod screen_implementation;
use std::error::Error;

const DEFAULT_ORDERS_PATH: &str = "helados.csv";

/// Funcion que se ocupa de parsear la entrada de terminal para tomar en consideracion la entrada. Devuelve un result y permite iniciar la conexion de pantalla
fn parse_args_for_screen() -> Result<String, String> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);

    let mut orders_path = DEFAULT_ORDERS_PATH.to_string();

    match args.len() {
        0 => {
            println!("[Screen] No arguments provided, default order to be used");
        }
        1 => {
            orders_path = args[0].to_owned();
            println!("[Screen] Orders file name given: {}", orders_path);
        }
        _ => {
            return Err("[Screen] Too many arguments".to_string());
        }
    }
    Ok(orders_path)
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let orders_path = parse_args_for_screen()?;
    screen_implementation::screen_connection_handler::start(orders_path)?;
    Ok(())
}
