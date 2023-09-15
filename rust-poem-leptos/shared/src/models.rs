use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHelloRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHelloResponse {
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct ClientToServerMessage {
    pub client_id: String,
    pub contents: ClientToServerMessageContents,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientToServerMessageContents {
    Message { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct ServerToClientMessage {
    pub contents: ServerToClientMessageContents,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerToClientMessageContents {
    Message { client_id: String, message: String },
    ClientJoined { client_id: String },
    ClientLeft { client_id: String },
}
