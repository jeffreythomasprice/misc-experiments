mod id;
mod timestamp;

pub use id::Id;
use serde::{Deserialize, Serialize};
pub use timestamp::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketClientToServerMessage {
    Hello { name: String },
    Message { id: Id, timestamp: Timestamp, payload: String },
}

impl WebsocketClientToServerMessage {
    pub fn new_message(message: String) -> Self {
        Self::Message {
            id: Id::new(),
            timestamp: Timestamp::now(),
            payload: message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketServerToClientMessage {
    Welcome {
        id: Id,
    },
    Message {
        id: Id,
        timestamp: Timestamp,
        sender: String,
        payload: String,
    },
}
