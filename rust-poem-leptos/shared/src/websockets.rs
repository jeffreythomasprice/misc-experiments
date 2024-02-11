use std::str::Utf8Error;

use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    Utf8(Utf8Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl Message {
    pub fn deserialize<T>(&self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        match self {
            Message::Text(msg) => Ok(serde_json::from_str(&msg)?),
            Message::Binary(msg) => {
                let msg = std::str::from_utf8(msg)?;
                Ok(serde_json::from_str(&msg)?)
            }
        }
    }

    pub fn serialize<T>(value: &T) -> Result<Message, Error>
    where
        T: Serialize,
    {
        let msg = serde_json::to_string(value)?;
        // TODO test binary messages
        Ok(Message::Text(msg))
    }
}
