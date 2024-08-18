//! Module with definitions of shop requests and responses

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ShopRequest {
    IceCreamOrder {
        flavors: Vec<String>,
        size: u32,
        screen_id: usize,
        screen_address: String
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ShopResponse {
    OrderResult { screen_id: usize, result: Result<(), String>, screen_address: String },
}
