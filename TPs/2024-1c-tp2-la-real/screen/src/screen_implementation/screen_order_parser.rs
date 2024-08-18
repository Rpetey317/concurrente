use std::fs::File;
use std::io::{BufRead, BufReader};

/// `ScreenOrderParser` parsea las líneas de un archivo CSV que representan órdenes de la pantalla y las devuelve como vectores de cadenas.
pub struct ScreenOrderParser {
    file_path: String,
}

impl ScreenOrderParser {
    /// Crea una nueva instancia de `ScreenOrderParser` con la ruta del archivo CSV especificada.
    ///
    /// # Parámetros
    ///
    /// * `file_path` - Ruta del archivo CSV a ser parseado.
    ///
    /// # Retorna
    ///
    /// Una nueva instancia de `ScreenOrderParser`.
    pub fn new(file_path: String) -> Self {
        ScreenOrderParser { file_path }
    }

    /// Retorna las órdenes parseadas del archivo CSV como vectores de cadenas.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` con un vector de vectores de cadenas en caso de éxito o un `String` en caso de cualquier error durante la lectura o el parseo del archivo.
    pub fn get_orders(&self) -> Result<Vec<Vec<String>>, String> {
        let file = File::open(&self.file_path).map_err(|error| error.to_string())?;
        let reader = BufReader::new(file);

        let mut orders: Vec<Vec<String>> = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|error| error.to_string())?;
            let order: Vec<String> = line
                .split(&[',', ':', ';'])
                .map(|s| s.trim().to_string())
                .collect();
            orders.push(order);
        }

        Ok(orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueba para verificar que el método `get_orders` retorna una lista vacía cuando el archivo está vacío.
    #[test]
    fn test_get_orders_empty_file() {
        let path = "./docs_ejemplo/test_empty_file.csv".to_owned();
        let parser = ScreenOrderParser::new(path);
        let result = parser.get_orders();
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), 0);
    }

    /// Prueba para verificar que el método `get_orders` procesa correctamente un archivo con una sola línea.
    #[test]
    fn test_get_orders_single_line() {
        let path = "./docs_ejemplo/test_single_line.csv".to_owned();
        let parser = ScreenOrderParser::new(path);
        let result = parser.get_orders();
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0], vec!["MEDIO", "crema"]);
    }

    /// Prueba para verificar que el método `get_orders` procesa correctamente un archivo con múltiples líneas.
    #[test]
    fn test_get_orders_multiple_lines() {
        let path = "./docs_ejemplo/test_multiple_line.csv".to_owned();
        let parser = ScreenOrderParser::new(path);
        let result = parser.get_orders();
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), 3);
        assert_eq!(orders[0], vec!["MEDIO", "crema"]);
        assert_eq!(orders[1], vec!["KILO", "frutilla", "dulce de leche"]);
        assert_eq!(orders[2], vec!["CUARTO", "menta", "vainilla"]);
    }
}
