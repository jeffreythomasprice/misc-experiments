pub mod websockets;

use std::num::TryFromIntError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_time::{SystemTimeError, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClicksResponse {
    pub clicks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientToServerChatMessage {
    // TODO message id
    #[serde(with = "date_format")]
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl ClientToServerChatMessage {
    pub fn new(msg: String) -> Result<Self, TimeError> {
        Ok(Self {
            timestamp: now_utc()?,
            message: msg,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToClientChatMessage {
    // TODO message id
    #[serde(with = "date_format")]
    pub timestamp: DateTime<Utc>,
    #[serde(with = "date_format")]
    pub message_timestamp: DateTime<Utc>,
    // TODO which sender
    pub message: String,
}

impl ServerToClientChatMessage {
    pub fn new(msg: &ClientToServerChatMessage) -> Result<Self, TimeError> {
        Ok(Self {
            timestamp: now_utc()?,
            message_timestamp: msg.timestamp,
            message: msg.message.clone(),
        })
    }
}

#[derive(Debug)]
pub enum TimeError {
    SystemTime(web_time::SystemTimeError),
    TryFromInt(TryFromIntError),
    DateTimeFromMillis,
}

pub fn now_utc() -> Result<DateTime<Utc>, TimeError> {
    let result = web_time::SystemTime::now();
    let result = result
        .duration_since(web_time::UNIX_EPOCH)
        .map_err(|e| TimeError::SystemTime(e))?;
    let result: i64 = result
        .as_millis()
        .try_into()
        .map_err(|e| TimeError::TryFromInt(e))?;
    DateTime::<Utc>::from_timestamp_millis(result).ok_or(TimeError::DateTimeFromMillis)
}

mod date_format {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&s)
            .map_err(|e| serde::de::Error::custom(e))?
            .to_utc())
    }
}
