use std::error::Error;

use ice_cream_robot::robot_handler::start_serving;

pub mod ice_cream_robot;

pub fn start() -> Result<(), Box<dyn Error>> {
    let mut initial_arguments: Vec<String> = std::env::args().collect();
    initial_arguments.remove(0); // no necesito el nombre del programa

    let arguments_length = initial_arguments.len();

    if arguments_length < 2 {
        println!("You need to pass the ice cream shop listening port, the screens listening port");
        return Err("You need to pass the ice cream shop listening port, the screens listening port".into());
    }

    let robot_to_robot_port: usize = match initial_arguments[0].parse() {
        Ok(robot_to_robot_port) => robot_to_robot_port,
        Err(_) => {
            println!("Couldn't convert, using default port for robot");
            10000
        }
    };

    let screens_port: usize = match initial_arguments[1].parse() {
        Ok(screens_port) => screens_port,
        Err(_) => {
            println!("Couldn't convert, using default port for screen");
            9000
        }
    };

    let result = start_serving(robot_to_robot_port, screens_port);
    result
}
