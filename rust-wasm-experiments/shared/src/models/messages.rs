use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericResponse {
    pub message: String,
}

impl GenericResponse {
    pub fn ok() -> Self {
        Self {
            message: "OK".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateClientResponse {
    pub id: String,
    pub token: String,
}

impl Debug for CreateClientResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateClientResponse")
            .field("id", &self.id)
            // .field("token", &self.token)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientWebsocketMessage {
    Authenticate { token: String },
}
