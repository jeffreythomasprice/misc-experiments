pub mod websockets;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClicksResponse {
    pub clicks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientToServerChatMessage {
    // TODO message id
    // TODO timestamp
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToClientChatMessage {
    // TODO message id
    // TODO timestamp
    // TODO which sender
    pub message: String,
}
