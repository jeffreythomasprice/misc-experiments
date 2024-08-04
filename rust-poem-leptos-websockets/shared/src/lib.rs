use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebsocketClientToServerMessage {
    Message(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebsocketServerToClientMessage {
    Message(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LogInRequest {
    pub username: String,
    pub password: String,
}
