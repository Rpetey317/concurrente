use std::sync::Arc;

use actix::prelude::*;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

use common::communication::shop_requests::{ShopRequest, ShopResponse};
use tracing::{debug, info, trace, warn};

use crate::shop_connection::stock_requester::{MakeOrder, StockRequester};

/// This actor is responsible for direct communication with the robots.
/// Manages message seraliazation, routing, and deserialization.
pub struct RobotCommunicator {
    writer: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
    requester: Addr<StockRequester>,
}

impl Actor for RobotCommunicator {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Robot disconnected.");
    }
}

impl RobotCommunicator {
    /// Creates a new RobotCommunicator actor to handle connections to the given stream.
    /// The read half of the stream should be added to the actor via `add_stream`.
    /// Any requests for stock will be forwarded to the given `StockRequester`.
    pub fn new(
        writer: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
        requester: Addr<StockRequester>,
    ) -> Self {
        RobotCommunicator { writer, requester }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct SendMessage {
    pub message: ShopResponse,
}

impl Handler<SendMessage> for RobotCommunicator {
    type Result = Result<(), String>;

    /// Sends a response back to robot.
    fn handle(&mut self, msg: SendMessage, _: &mut Self::Context) -> Result<(), String> {
        let msg = serde_json::to_string(&msg.message).map_err(|err| err.to_string())? + "\n";
        let writer = self.writer.clone();
        actix::spawn(async move {
            trace!("Sending message: {:?}", msg);
            match writer.lock().await.write_all(msg.as_bytes()).await {
                Ok(()) => debug!("Message sent to robot:\n{}", msg),
                Err(e) => warn!("Error sending message to robot: {:?}", e),
            };
            drop(writer);
            trace!("Message sent, lock dropped");
        });
        Ok(())
    }
}

impl StreamHandler<Result<String, std::io::Error>> for RobotCommunicator {
    /// Handles incoming messages from the robot and forwards them as necessary.
    fn handle(&mut self, msg: Result<String, std::io::Error>, ctx: &mut Context<Self>) {
        if let Ok(msg) = msg {
            if let Ok(request) = serde_json::from_str::<ShopRequest>(&msg) {
                #[allow(unreachable_patterns)]
                match request {
                    ShopRequest::IceCreamOrder {
                        size: amount,
                        ref flavors,
                        screen_id: index,
                        screen_address: addr,
                    } => {
                        trace!("Received request from robot: size {}, no. of flavors {} (idx {}, addr {})", amount, flavors.len(), index, addr);

                        self.requester.do_send(MakeOrder {
                            return_addr: ctx.address(),
                            size: amount,
                            flavors: flavors.clone(),
                            screen_index: index,
                            screen_address: addr.clone(),
                        });
                    }
                    _ => {
                        warn!(
                            "Received unsuported type of request from robot: {:?}",
                            request
                        )
                    }
                }
            }
        }
    }
}
