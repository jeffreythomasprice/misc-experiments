use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebsocketClientToServerMessage {
    Message(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebsocketServerToClientMessage {
    Message(String),
}
