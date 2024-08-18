use std::{process::exit, thread::JoinHandle};

/// `UserInput` es una estructura que maneja la entrada del usuario desde la terminal.
pub struct UserInput {}

impl UserInput {
    /// Inicia un hilo que escucha la entrada del usuario y cierra el programa cuando se ingresa `q`.
    ///
    /// # Retornos
    ///
    /// Devuelve un `JoinHandle` que puede ser usado para esperar la finalización del hilo.
    pub fn start_user_input() -> JoinHandle<Result<(), String>> {
        std::thread::spawn(move || -> Result<(), String> {
            println!("UserInput started");
            let mut reader = std::io::stdin().lines();

            while let Some(Ok(line)) = reader.next() {
                if line == "q" {
                    exit(0);
                } else {
                    println!("Unknown command. use q");
                }
            }
            Ok(())
        })
    }
}
