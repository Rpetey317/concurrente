use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    
    ScreenToRobotOrder {
        index: usize,
        flavors: Vec<String>,
        size: u32,
    },
    ScreenToRobotAskLeader {}
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    RobotToScreenResult {
        index: usize,
        result: Result<(), String>
    },
    RobotToScreenLeaderPort {
        leader_port: usize
    }
}
