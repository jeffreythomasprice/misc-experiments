pub mod websockets;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClicksResponse {
    pub clicks: u64,
}

// TODO make a different server and client message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message: String,
}
