use common::communication::screen_robot_communication::Request;

/// Modela una orden individual, con su índice, estado y detalles.
/// Permite conocer su estado, serializa la información para la comunicación
/// y maneja el resultado en función del estado recibido de los robots.
pub struct ScreenWorkingOrder {
    order: common::communication::screen_robot_communication::Request,
    state: OrderState,
}

/// Enumerates the possible states of a `ScreenWorkingOrder`.
#[derive(Clone, Debug)]
pub enum OrderState {
    Pending,
    Completed,
    Failed,
}

impl ScreenWorkingOrder {
    /// Crea una nueva `ScreenWorkingOrder` con el índice, tamaño y sabores especificados.
    ///
    /// # Argumentos
    ///
    /// * `index` - Índice de la orden.
    /// * `size` - Tamaño de la orden en formato de texto (e.g., "KILO", "MEDIO", "CUARTO").
    /// * `flavours` - Vector de sabores para la orden.
    ///
    /// # Retornos
    ///
    /// Devuelve una nueva instancia de `ScreenWorkingOrder`.
    pub fn new(index: usize, size: String, flavours: Vec<String>) -> Self {
        let grams = match size.as_str() {
            "KILO" => 1000,
            "MEDIO" => 500,
            "CUARTO" => 250,
            _ => 0,
        };
        ScreenWorkingOrder {
            order: Request::ScreenToRobotOrder {
                index: index.to_owned(),
                flavors: flavours.to_owned(),
                size: grams,
            },
            state: OrderState::Pending,
        }
    }

    /// Serializa la información de la orden en un formato que el robot puede entender.
    ///
    /// # Retornos
    ///
    /// Devuelve una cadena de texto que representa la orden serializada lista para ser enviada.
    pub fn get_serialized_order_information(&self) -> String {
        let mut order_to_send = String::new();
        if let Ok(serialized) = serde_json::to_string(&self.order) {
            order_to_send.push_str(&serialized.to_string());
        }
        order_to_send
    }

    /// Marca una orden como exitosa.
    pub fn succesfull_order(&mut self) {
        self.state = OrderState::Completed;
        match &self.order {
            Request::ScreenToRobotOrder {
                index,
                flavors: _,
                size: _,
            } => {
                println!("Order #{} was succesfully finished!", index);
            }
            Request::ScreenToRobotAskLeader {} => {}
        }
    }

    /// Marca una orden como fallida.
    pub fn unsuccesfull_order(&mut self) {
        self.state = OrderState::Failed;
        match &self.order {
            Request::ScreenToRobotOrder {
                index,
                flavors: _,
                size: _,
            } => {
                println!(
                    "Order #{} couldn't be finished, it won't be charged.",
                    index
                );
            }
            Request::ScreenToRobotAskLeader {} => {}
        }
    }

    pub fn valid(&self) -> bool {
        match &self.order {
            Request::ScreenToRobotOrder {
                index: _,
                flavors: _,
                size,
            } => *size != 0,
            Request::ScreenToRobotAskLeader {} => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_flavour_amount_kilo() {
        let working_order = ScreenWorkingOrder::new(
            1,
            String::from("KILO"),
            vec![String::from("Chocolate"), String::from("Vanilla")],
        );
        let grams;
        match working_order.order {
            Request::ScreenToRobotOrder {
                index: _,
                flavors: _,
                size,
            } => {
                grams = size;
            }
            Request::ScreenToRobotAskLeader {} => grams = 0,
        }
        assert_eq!(grams, 1000);
    }

    #[test]
    fn test_get_flavour_amount_medio() {
        let working_order = ScreenWorkingOrder::new(
            2,
            String::from("MEDIO"),
            vec![String::from("Frutilla"), String::from("Menta")],
        );
        let grams;
        match working_order.order {
            Request::ScreenToRobotOrder {
                index: _,
                flavors: _,
                size,
            } => {
                grams = size;
            }
            _ => grams = 25000, //hara que el test falle
        }
        assert_eq!(grams, 500);
    }

    #[test]
    fn test_get_flavour_amount_cuarto() {
        let working_order = ScreenWorkingOrder::new(
            3,
            String::from("CUARTO"),
            vec![String::from("Dulce de leche"), String::from("Menta")],
        );
        let grams;
        match working_order.order {
            Request::ScreenToRobotOrder {
                index: _,
                flavors: _,
                size,
            } => {
                grams = size;
            }
            _ => grams = 25000, //hara que el test falle
        }
        assert_eq!(grams, 250);
    }

    #[test]
    fn test_get_flavour_amount_invalid() {
        let working_order = ScreenWorkingOrder::new(
            4,
            String::from("CUCURUCHO"),
            vec![String::from("Limon"), String::from("Pistacho")],
        );
        let grams;
        match working_order.order {
            Request::ScreenToRobotOrder {
                index: _,
                flavors: _,
                size,
            } => {
                grams = size;
            }
            _ => grams = 25000,
        }
        assert_eq!(grams, 0);
    }

    #[test]
    fn test_get_serialized_order_information() {
        let order = ScreenWorkingOrder::new(
            5,
            String::from("KILO"),
            vec![String::from("Chocolate"), String::from("Vanilla")],
        );
        let mut expected = String::new();
        if let Ok(serialized) = serde_json::to_string(&Request::ScreenToRobotOrder {
            index: 5,
            flavors: vec![String::from("Chocolate"), String::from("Vanilla")],
            size: 1000,
        }) {
            expected.push_str(&serialized.to_string());
        }
        assert_eq!(order.get_serialized_order_information(), expected);
    }
}
