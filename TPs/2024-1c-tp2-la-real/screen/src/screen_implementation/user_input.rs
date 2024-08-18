/// Permite cerrar la pantalla a voluntad con `q` de entrada por terminal.
use std::{process::exit, thread::JoinHandle};

/// `UserInput` es una estructura que maneja la entrada del usuario desde la terminal.
pub struct UserInput {}

impl UserInput {
    /// Inicia un hilo que escucha la entrada del usuario y cierra el programa cuando se ingresa `q` o `Q`.
    ///
    /// # Retornos
    ///
    /// Devuelve un `JoinHandle` que puede ser usado para esperar la finalización del hilo.
    ///
    /// # Ejemplo
    ///
    /// ```text
    /// let handle = UserInput::start_user_input();
    /// ```
    ///
    /// En este ejemplo, se inicia un nuevo hilo que escucha la entrada del usuario desde la terminal. Si el usuario ingresa `q` o `Q`, el programa se cerrará.
    pub fn start_user_input() -> JoinHandle<Result<(), String>> {
        std::thread::spawn(move || -> Result<(), String> {
            println!("UserInput started");
            let mut reader = std::io::stdin().lines();

            while let Some(Ok(line)) = reader.next() {
                if line.to_lowercase() == "q" {
                    exit(0);
                } else {
                    println!("Unknown command. To exit use Q or q");
                }
            }
            Ok(())
        })
    }
}
