use crate::screen_implementation::screen_payment_system::ScreenPaymentSystem;
use crate::screen_implementation::screen_to_robot_connection::*;
use crate::screen_implementation::screen_working_order::ScreenWorkingOrder;
use actix::prelude::*;
use common::communication::screen_robot_communication::Request;
use std::collections::HashMap;

use super::screen_to_robot_connection::ScreenRobotConnection;

/// Actor principal de las pantallas. Se ocupa de gestionar las ordenes desde que son recibidas hasta que se terminan.
pub struct ScreenOrderManager {
    orders: Vec<Vec<String>>,
    sent_orders: HashMap<usize, ScreenWorkingOrder>,
    screen_connection: Option<Addr<ScreenRobotConnection>>,
    payment_system: ScreenPaymentSystem,
    total_order_count: usize,
    sent_indexes: Vec<usize>,
    sent_all_orders: bool,
    currently_connected_port: Option<usize>,
}

impl Actor for ScreenOrderManager {
    type Context = Context<Self>;
}

impl ScreenOrderManager {
    /// Crea un nuevo `ScreenOrderManager` con las órdenes proporcionadas y una conexión a un `ScreenRobotConnection`.
    ///
    /// # Parámetros
    ///
    /// * `screen_connection`: Dirección del `ScreenRobotConnection`.
    /// * `orders`: Vector de órdenes a procesar.
    ///
    pub fn new_manager_with_orders(orders: Vec<Vec<String>>) -> Self {
        println!("[ScreenOrderManager] Created!");
        ScreenOrderManager {
            sent_orders: HashMap::new(),
            total_order_count: orders.len(),
            orders,
            sent_indexes: Vec::new(),
            screen_connection: None,
            payment_system: ScreenPaymentSystem::new(),
            currently_connected_port: None,
            sent_all_orders: false,
        }
    }

    /// Cierra una orden que pudo ser completada.
    ///
    /// # Parámetros
    ///
    /// * `index`: Índice de la orden.
    /// * `recived_order`: Información de la orden recibida para mostrar en caso de error.
    fn validate_completed_order(&mut self, index: &str) {
        if let Ok(index) = index.parse::<usize>() {
            if let Some(working_order) = self.sent_orders.get_mut(&index) {
                working_order.succesfull_order();
            }
        } else {
            println!(
                "[ScreenOrderManager] The received order ({}) is not in my records",
                index
            );
        }
    }

    /// Cierra una orden que no pudo ser completada.
    ///
    /// # Parámetros
    ///
    /// * `index`: Índice de la orden.
    /// * `recived_order`: Información de la orden recibida para mostrar en caso de error.
    fn validate_rejected_order(&mut self, index: &str) {
        if let Ok(index) = index.parse::<usize>() {
            if let Some(working_order) = self.sent_orders.get_mut(&index) {
                working_order.unsuccesfull_order();
            }
        } else {
            println!(
                "[ScreenOrderManager] The received order ({}) is not in my records",
                index
            );
        }
    }

    fn get_current_order(&mut self) -> usize{
        let vec = self.sent_indexes.clone();
        if vec.is_empty() {
            return 0;
        }
        *(vec.iter().max().unwrap())
    }
}

/// Mensaje para iniciar el envío de órdenes.
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartSendingOrders {}

impl Handler<StartSendingOrders> for ScreenOrderManager {
    type Result = ();

    fn handle(&mut self, _msg: StartSendingOrders, ctx: &mut Self::Context) {
        if !self.sent_all_orders {
            let _ = ctx
                .address()
                .try_send(SendOrder {
                    numero_de_orden: self.get_current_order(),
                })
                .map_err(|error| error.to_string());
        }
    }
}

/// Mensaje para enviar una orden específica.
///
/// # Parámetros
///
/// * `numero_de_orden`: Índice de la orden a enviar.
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendOrder {
    pub numero_de_orden: usize,
}

impl Handler<SendOrder> for ScreenOrderManager {
    type Result = ();

    fn handle(&mut self, msg: SendOrder, ctx: &mut Self::Context) {
        if self.total_order_count <= msg.numero_de_orden {
            self.sent_all_orders = true;
            return;
        }
        let mut working_order = ScreenWorkingOrder::new(
            msg.numero_de_orden,
            self.orders[msg.numero_de_orden][0].clone(),
            self.orders[msg.numero_de_orden][1..].to_vec(),
        );
        self.sent_indexes.push(msg.numero_de_orden);
        if working_order.valid() {
            if self.payment_system.attempt_payment_capture() {
                if let Some(screen_connection) = &self.screen_connection {
                    let _ = screen_connection
                    .try_send(SendMessageThroughSocket {
                        message_to_send_through_socket: working_order
                        .get_serialized_order_information(),
                    })
                    .map_err(|err| err.to_string());
                println!("[ScreenOrderManager] SendOrder #{}", msg.numero_de_orden);
                println!(
                    "Sent order: {}",
                    working_order.get_serialized_order_information()
                );
                self.sent_orders.insert(msg.numero_de_orden, working_order);
                }
            } else {
                println!(
                    "The order: [{}], was rejected due to payment capture failure.",
                    working_order.get_serialized_order_information()
                );
                working_order.unsuccesfull_order();
                let _ = ctx
                .address()
                .try_send(SendOrder {
                    numero_de_orden: self.get_current_order() + 1,
                })
                .map_err(|error| error.to_string());
            }
        } else {
            println!("The order attempted was invalid");
            let _ = ctx
                .address()
                .try_send(SendOrder {
                    numero_de_orden: self.get_current_order() + 1,
                })
                .map_err(|error| error.to_string());
        }
    }
}

/// Mensaje recibido del robot una vez que se ha procesado una orden. La orden se cierra según corresponda.
///
/// # Parámetros
///
/// * `result`: Resultado del procesamiento de la orden.
/// * `index`: Índice de la orden procesada.
#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ReturnedOrderFromShop {
    pub result: Result<(), String>,
    pub index: u32,
}

impl Handler<ReturnedOrderFromShop> for ScreenOrderManager {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: ReturnedOrderFromShop, ctx: &mut Self::Context) -> Self::Result {
        println!("[ScreenOrderManager] ReturnedOrderFromShop: #{}", msg.index);
        match msg.result {
            Ok(_) => {
                self.validate_completed_order(&msg.index.to_string());
            }
            Err(_) => {
                self.validate_rejected_order(&msg.index.to_string());
            }
        }
        if self.get_current_order() < self.total_order_count{
        ctx.address()
            .try_send(SendOrder {
                numero_de_orden: (self.get_current_order() + 1),
            })
            .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

/// Mensaje para añadir una conexión `ScreenRobotConnection` al `ScreenOrderManager`.
///
/// # Parámetros
///
/// * `addr_screen_robot_connection`: Dirección de la conexión `ScreenRobotConnection`.
/// * `port`: Puerto conectado.
#[derive(Message, Debug)]
#[rtype(result = "Result<(), String>")]
pub struct AddScreenRobotConnection {
    pub addr_screen_robot_connection: Addr<ScreenRobotConnection>,
    pub port: usize,
}

impl Handler<AddScreenRobotConnection> for ScreenOrderManager {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: AddScreenRobotConnection, _: &mut Context<Self>) -> Self::Result {
        self.screen_connection = Some(msg.addr_screen_robot_connection.clone());
        self.currently_connected_port = Some(msg.port);
        let message = serde_json::to_string(&Request::ScreenToRobotAskLeader {})
            .map_err(|error| error.to_string())?;
        msg.addr_screen_robot_connection
            .try_send(SendMessageThroughSocket {
                message_to_send_through_socket: message,
            })
            .map_err(|error| error.to_string())
    }
}

/// Mensaje para indicar quién es el líder del servidor.
///
/// # Parámetros
///
/// * `leader_server_id`: ID del servidor líder.
#[derive(Message, Debug)]
#[rtype(result = "Result<(), String>")]
pub struct LeaderMessage {
    pub leader_server_id: usize,
}

impl Handler<LeaderMessage> for ScreenOrderManager {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: LeaderMessage, _: &mut Context<Self>) -> Self::Result {
        if let Some(curr_server_id) = &mut self.currently_connected_port {
            *curr_server_id = msg.leader_server_id;
        }

        Err("Current port not set, cannot connect to leader.".to_owned())
    }
}
