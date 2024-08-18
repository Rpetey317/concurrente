//! Module containing all the network connection logic for the shop.
//! - `RobotListener`: Listens incoming and sets up new connections.
//! - `RobotCommunicator`: Handles network communication with a single robot.
//! - `StockRequester`: Bridges communications between the shop and the socket.

pub mod robot_communicator;
pub mod robot_listener;
pub mod stock_requester;
