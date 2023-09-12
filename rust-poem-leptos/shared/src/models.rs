use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHelloRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHelloResponse {}
