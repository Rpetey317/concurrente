use std::{collections::HashMap, net::SocketAddr};

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};

use crate::ice_cream_robot::robot_to_robot_connection::{SendLeaderSelected, SendStartElections};

use super::{
    ice_cream_shop_connection::{IceCreamShopConnection, RequestIceCreamFlavor},
    robot_to_robot_connection::{RobotToRobotConnection, SendMessageThroughSocketToRobot},
    robot_to_screen_connection::{
        CloseConnectionFromScreen, RobotToScreenConnection, SendMessageThroughSocketToScreen,
        SuccessForScreenMessage,
    },
};
use common::communication::shop_requests::ShopResponse;
use common::communication::{
    robot_to_robot_communication::RobotRequest, screen_robot_communication::Response,
};

/// `ManageRobotConnection` es un actor que gestiona las conexiones entre el robot, la tienda de helados y las pantallas.
///
/// Este actor es responsable de mantener las conexiones activas con otros actores y manejar los mensajes que recibe.
/// También se encarga de la gestión de las órdenes entre la tienda de helados y las pantallas.
pub struct ManageRobotConnection {
    robot_to_ice_cream_shop_connection: Option<Addr<IceCreamShopConnection>>,
    robot_to_screen_connections: HashMap<String, Addr<RobotToScreenConnection>>,
    robot_to_robot_connections: HashMap<usize, Addr<RobotToRobotConnection>>,

    robot_to_robot_id: usize,
    robot_to_robot_leader_id: Option<usize>,

    robot_to_screen_id: usize,
    robot_to_screen_leader_id: Option<usize>,
}

impl ManageRobotConnection {

    /// Crea una nueva instancia de `ManageRobotConnection`.
    ///
    /// # Argumentos
    ///
    /// * `robot_to_robot_id` - ID del robot actual en la red de robots.
    /// * `robot_to_screen_id` - ID de la pantalla actual con la que el robot está conectado.
    ///
    /// # Retorna
    ///
    /// Una nueva instancia de `ManageRobotConnection` con las conexiones vacías y sin líder asignado.
    pub fn new(robot_to_robot_id: usize, robot_to_screen_id: usize) -> Self {
        Self {
            robot_to_ice_cream_shop_connection: None,
            robot_to_screen_connections: HashMap::new(),

            robot_to_screen_id,
            robot_to_screen_leader_id: None,

            robot_to_robot_id,
            robot_to_robot_leader_id: None,
            robot_to_robot_connections: HashMap::new(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje utilizado para agregar una conexión con la tienda de helados.
pub struct AddIceCreamShopConnectionMessage {
    pub robot_to_ice_cream_shop_connection: Addr<IceCreamShopConnection>,
}

impl Handler<AddIceCreamShopConnectionMessage> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Actualiza la conexión del robot con la tienda de helados.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `AddIceCreamShopConnectionMessage` que contiene la nueva conexión.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación fue exitosa o no.
    fn handle(
        &mut self,
        msg: AddIceCreamShopConnectionMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.robot_to_ice_cream_shop_connection = Some(msg.robot_to_ice_cream_shop_connection);
        Ok(())
    }
}

impl Actor for ManageRobotConnection {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje utilizado para informar al `ManageRobotConnection` sobre una orden resuelta de la tienda de helados.
pub struct SolvedOrderFromIceCreamShop {
    pub message: String,
}


impl Handler<SolvedOrderFromIceCreamShop> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Procesa el resultado de una orden de la tienda de helados y lo envía a la pantalla correspondiente.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `SolvedOrderFromIceCreamShop` que contiene el resultado de la orden.
    /// * `ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación fue exitosa o no.
    fn handle(&mut self, msg: SolvedOrderFromIceCreamShop, ctx: &mut Self::Context) -> Self::Result {
        match serde_json::from_str(&msg.message).map_err(|error| error.to_string())? {
            ShopResponse::OrderResult {
                screen_id,
                result,
                screen_address,
            } => {
                let message = serde_json::to_string(&Response::RobotToScreenResult {
                    index: screen_id,
                    result,
                })
                .map_err(|error| error.to_string())?;
                ctx.address()
                    .try_send(SuccessInProcessingOrder {
                        screen_address,
                        message,
                    })
                    .map_err(|error| error.to_string())
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje utilizado para informar al `ManageRobotConnection` sobre una orden recibida de una pantalla.
pub struct ReceivedOrderFromScreen {
    pub message: String,
}

impl Handler<ReceivedOrderFromScreen> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Envía una solicitud a la tienda de helados para obtener el sabor del helado basado en la orden recibida.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `ReceivedOrderFromScreen` que contiene la orden de la pantalla.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación fue exitosa o no.
    fn handle(&mut self, msg: ReceivedOrderFromScreen, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(robot_to_ice_cream_shop_connection) = &self.robot_to_ice_cream_shop_connection {
            robot_to_ice_cream_shop_connection
                .try_send(RequestIceCreamFlavor {
                    message: msg.message,
                })
                .map_err(|error| error.to_string())
        } else {
            Ok(())
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje utilizado para informar al `ManageRobotConnection` sobre el éxito en el procesamiento de una orden.
pub struct SuccessInProcessingOrder {
    message: String,
    screen_address: String,
}

impl Handler<SuccessInProcessingOrder> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Si el robot es el líder, envía un mensaje de éxito a la pantalla correspondiente.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `SuccessInProcessingOrder` que contiene la información sobre el éxito en el procesamiento de la orden.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación fue exitosa o no.
    fn handle(&mut self, msg: SuccessInProcessingOrder, _: &mut Self::Context) -> Self::Result {
        if self.robot_to_robot_id == self.robot_to_robot_leader_id.unwrap() {
            if let Some(screen) = self.robot_to_screen_connections.get(&msg.screen_address) {
                return screen
                    .try_send(SuccessForScreenMessage {
                        message_to_send_through_socket: msg.message,
                    })
                    .map_err(|error| error.to_string());
            }
        }
        Ok(())
    }
}


#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para agregar una nueva conexión entre el robot y una pantalla.
pub struct AddRobotToScreenConnectionMessage {
    pub address: SocketAddr,
    pub robot_to_screen_connection: Addr<RobotToScreenConnection>,
}

impl Handler<AddRobotToScreenConnectionMessage> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método agrega una nueva conexión de pantalla al mapa de conexiones de pantalla del robot.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `AddRobotToScreenConnectionMessage` que contiene la dirección de la pantalla y la conexión.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación de agregar la conexión fue exitosa.
    fn handle(
        &mut self,
        msg: AddRobotToScreenConnectionMessage,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.robot_to_screen_connections.insert(
            msg.address.to_string(),
            msg.robot_to_screen_connection.clone(),
        );
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para eliminar una conexión entre el robot y una pantalla.
pub struct RemoveRobotToScreenConnection {
    pub stream_addr: SocketAddr,
}


impl Handler<RemoveRobotToScreenConnection> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método elimina una conexión de pantalla del mapa de conexiones de pantalla del robot.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `RemoveRobotToScreenConnection` que contiene la dirección de la pantalla a eliminar.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación de eliminar la conexión fue exitosa.
    fn handle(
        &mut self,
        msg: RemoveRobotToScreenConnection,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.robot_to_screen_connections
            .remove(&msg.stream_addr.to_string());
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para solicitar el ID del líder de la pantalla.
pub struct AskLeaderMessage {
    pub robot_to_screen_connection_address: Addr<RobotToScreenConnection>,
}

impl Handler<AskLeaderMessage> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método envía una solicitud a una conexión de pantalla para obtener el ID del líder de la pantalla.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `AskLeaderMessage` que contiene la dirección del actor `RobotToScreenConnection`.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la solicitud fue enviada exitosamente.
    fn handle(&mut self, msg: AskLeaderMessage, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(robot_to_screen_leader_id) = self.robot_to_screen_leader_id {
            let message = serde_json::to_string(&Response::RobotToScreenLeaderPort {
                leader_port: robot_to_screen_leader_id,
            })
            .map_err(|error| error.to_string())?;
            msg.robot_to_screen_connection_address
                .try_send(SendMessageThroughSocketToScreen {
                    message_to_send_through_socket: message,
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(),String>")]
/// Mensaje para iniciar una elección de líder entre los robots.
pub struct StartLeaderElection {}

impl Handler<StartLeaderElection> for ManageRobotConnection {
    type Result = Result<(), String>;


    /// Este método inicia una elección de líder entre los robots, seleccionando al robot con el ID más alto como el nuevo líder.
    ///
    /// # Argumentos
    ///
    /// * `_msg` - El mensaje `StartLeaderElection` para iniciar el proceso de elección.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la elección fue iniciada exitosamente.
    fn handle(&mut self, _msg: StartLeaderElection, _ctx: &mut Self::Context) -> Self::Result {
        println!("Let the elections for a leader begin!");
        if let Some(max_robot_to_robot_id) = self.robot_to_robot_connections.keys().max() {
            if max_robot_to_robot_id > &self.robot_to_robot_id {
                let robot_to_robot_connection = self
                    .robot_to_robot_connections
                    .get(max_robot_to_robot_id)
                    .ok_or(format!(
                        "This should never happen, a robot that didn't exists was elected leader :O, {}",
                        max_robot_to_robot_id
                    ))?;
                return robot_to_robot_connection
                    .try_send(SendStartElections {})
                    .map_err(|error| error.to_string());
            }
        };

        println!("L'État, c'est moi. I'm the new robot leader {}", self.robot_to_robot_id);

        self.robot_to_robot_leader_id = Some(self.robot_to_robot_id);
        self.robot_to_screen_leader_id = Some(self.robot_to_screen_id);
        for (_, robot_to_robot_connection) in self.robot_to_robot_connections.iter() {
            robot_to_robot_connection
                .try_send(SendLeaderSelected {
                    robot_to_robot_id: self.robot_to_robot_id,
                    robot_to_screen_id: self.robot_to_screen_id,
                })
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(),String>")]
/// Mensaje para notificar al `ManageRobotConnection` que un nuevo líder ha sido seleccionado.
pub struct LeaderSelected {
    pub robot_to_robot_leader_id: usize,
    pub robot_to_screen_leader_id: usize,
}

impl Handler<LeaderSelected> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método actualiza el ID del líder para los robots y la pantalla, y cierra todas las conexiones de pantalla.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `LeaderSelected` que contiene los IDs de los líderes de los robots y de la pantalla.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la actualización del líder y el cierre de conexiones fueron exitosos.
    fn handle(&mut self, msg: LeaderSelected, _ctx: &mut Self::Context) -> Self::Result {
        self.robot_to_robot_leader_id = Some(msg.robot_to_robot_leader_id);
        self.robot_to_screen_leader_id = Some(msg.robot_to_screen_leader_id);

        for robot_to_screen_connection in self.robot_to_screen_connections.values() {
            robot_to_screen_connection
                .try_send(CloseConnectionFromScreen {})
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(),String>")]
///Mensaje para notificar al `ManageRobotConnection` sobre la muerte de un robot.
pub struct ElectionForDeadRobot {
    pub closed_server_id: usize,
}

impl Handler<ElectionForDeadRobot> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método actualiza la lista de conexiones de robots para eliminar la conexión del robot que ha cerrado el servidor.
    /// Si el robot que ha muerto era el líder, se inicia un proceso para elegir un nuevo líder.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `ElectionForDeadRobot` que contiene el ID del robot muerto.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la eliminación del robot y el posible inicio de una nueva elección de líder fueron exitosos.
    fn handle(&mut self, msg: ElectionForDeadRobot, _ctx: &mut Self::Context) -> Self::Result {
        self.robot_to_robot_connections
            .remove(&msg.closed_server_id)
            .ok_or(format!("A not existing robot died??? {}", msg.closed_server_id))?;

        if let Some(leader_id) = self.robot_to_robot_leader_id {
            if leader_id != msg.closed_server_id {
                return Ok(());
            }
            println!("The King died! Don't worry, long live the King! Starting elections for a dead robot leader ");
            self.robot_to_robot_leader_id = None;
            if let Some(max_robot_to_robot_id) = self.robot_to_robot_connections.keys().max() {
                if let Some(min_robot_to_robot_id) = self.robot_to_robot_connections.keys().min() {
                    if &self.robot_to_robot_id < min_robot_to_robot_id {
                        let message = serde_json::to_string(&RobotRequest::StartElection {})
                            .map_err(|error| error.to_string())?;
                        self.robot_to_robot_connections
                            .get(max_robot_to_robot_id)
                            .ok_or(format!("The max leader didn't exist? {}", max_robot_to_robot_id))?
                            .try_send(SendMessageThroughSocketToRobot {
                                message_to_send_through_socket: message,
                            })
                            .map_err(|error| error.to_string())?;
                    }
                }
                return Ok(());
            };
            self.robot_to_robot_leader_id = Some(self.robot_to_robot_id);
        }
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para agregar una nueva conexión entre robots.
pub struct AddRobotToRobotConnectionMessage {
    pub robot_to_robot_id: Option<usize>,
    pub robot_to_robot_connection_address: Addr<RobotToRobotConnection>,
}

impl Handler<AddRobotToRobotConnectionMessage> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método agrega una nueva conexión entre robots al mapa de conexiones de robots del robot actual y envía una solicitud para obtener información del robot.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `AddRobotToRobotConnectionMessage` que contiene el ID del robot y la dirección de la conexión.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación de agregar la conexión y enviar la solicitud de información fue exitosa.
    fn handle(
        &mut self,
        msg: AddRobotToRobotConnectionMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Some(robot_to_robot_id) = msg.robot_to_robot_id {
            self.robot_to_robot_connections.insert(
                robot_to_robot_id,
                msg.robot_to_robot_connection_address.clone(),
            );
        }

        let message = serde_json::to_string(&RobotRequest::GetMyInformation {
            robot_to_robot_id: self.robot_to_robot_id,
        })
        .map_err(|error| error.to_string())?;

        msg.robot_to_robot_connection_address
            .try_send(SendMessageThroughSocketToRobot {
                message_to_send_through_socket: message,
            })
            .map_err(|error| error.to_string())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
/// Mensaje para registrar una nueva conexión entre robots.
pub struct RegisterRobotToRobotConnectionMessage {
    pub robot_to_robot_id: usize,
    pub robot_to_robot_connection: Addr<RobotToRobotConnection>,
}

impl Handler<RegisterRobotToRobotConnectionMessage> for ManageRobotConnection {
    type Result = Result<(), String>;

    /// Este método agrega una nueva conexión entre robots al mapa de conexiones de robots del robot actual.
    ///
    /// # Argumentos
    ///
    /// * `msg` - El mensaje `RegisterRobotToRobotConnectionMessage` que contiene el ID del robot y la dirección de la conexión.
    /// * `_ctx` - El contexto del actor `ManageRobotConnection`.
    ///
    /// # Retorna
    ///
    /// Un `Result` que indica si la operación de registrar la conexión fue exitosa.
    fn handle(
        &mut self,
        msg: RegisterRobotToRobotConnectionMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.robot_to_robot_connections
            .insert(msg.robot_to_robot_id, msg.robot_to_robot_connection.clone());

        Ok(())
    }
}
