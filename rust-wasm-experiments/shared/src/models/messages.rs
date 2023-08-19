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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClientResponse {
    pub id: String,
}
