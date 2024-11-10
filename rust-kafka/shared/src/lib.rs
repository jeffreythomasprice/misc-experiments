mod id;
mod timestamp;

use std::{str::FromStr, time::SystemTime};

use chrono::Utc;
use id::Id;
use serde::{de::Visitor, Deserialize, Serialize};
use timestamp::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Id,
    pub timestamp: Timestamp,
    pub sender: String,
    pub payload: String,
}

impl Message {
    pub fn new(sender: String, payload: String) -> Self {
        Self {
            id: Id::new(),
            timestamp: Timestamp::now(),
            sender,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketClientToServerMessage {
    Hello { name: String },
    Message(Message),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketServerToClientMessage {
    Welcome { id: String },
    Message(Message),
}
