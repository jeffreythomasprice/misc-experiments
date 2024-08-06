use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketClientToServerMessage {
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketServerToClientMessage {
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LogInRequest {
    pub username: String,
    pub password: String,
}
