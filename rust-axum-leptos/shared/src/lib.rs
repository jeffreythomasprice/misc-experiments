use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketClientToServerMessage {
    // TODO real messages
    Placeholder(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketServerToClientMessage {
    // TODO real messages
    Placeholder(String),
}
