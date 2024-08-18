use crate::shop_connection::{robot_communicator::RobotCommunicator, stock_requester::MakeOrder};
use actix::Addr;
use uuid::Uuid;

/// Struct encapsulating all information related to an order.
#[derive(Debug)]
pub struct ActiveOrder {
    /// Unique identifier for this order
    pub id: Uuid,
    /// Address to send the response to
    pub requester: Addr<RobotCommunicator>,
    /// Number of flavors to order
    pub flavors_to_order: u32,
    /// Flavors that have been ordered and their amounts
    pub flavors_ordered: Vec<(String, u32)>,
    /// Index of the screen that made the order
    pub screen_index: usize,
    /// Address of the screen that made the order
    pub screen_address: String,
}

impl ActiveOrder {
    /// Creates a new active order from a `MakeOrder` message.
    pub fn from_message(msg: &MakeOrder) -> Self {
        ActiveOrder {
            id: Uuid::new_v4(),
            requester: msg.return_addr.clone(),
            flavors_to_order: msg.flavors.len() as u32,
            flavors_ordered: Vec::new(),
            screen_index: msg.screen_index,
            screen_address: msg.screen_address.clone(),
        }
    }

    /// Checks if the order is done (i.e. all flavors were ordered).
    pub fn is_done(&self) -> bool {
        self.flavors_to_order == self.flavors_ordered.len() as u32
    }
}
