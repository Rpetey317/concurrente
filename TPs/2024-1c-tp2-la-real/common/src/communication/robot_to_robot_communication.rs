use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum RobotRequest {
    
    GetMyInformation {
        robot_to_robot_id: usize,
    },
    StartElection {

    },
    LeaderSelected {
        robot_to_robot_leader_id: usize,
        robot_to_screen_leader_id: usize,
    }
}

#[derive(Serialize, Deserialize)]
pub enum RobotResponse {
    RobotToScreenResult {
        index: usize,
        result: Result<(), String>
    },
    RobotToScreenLeaderPort {
        leader_port: usize
    }
}
