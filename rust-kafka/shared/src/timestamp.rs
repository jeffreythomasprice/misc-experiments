use std::{str::FromStr, time::SystemTime};

use chrono::{DateTime, Utc};
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Timestamp {
        Timestamp(Utc::now())
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
    }
}

struct TimestampVisitor {}

impl<'de> Visitor<'de> for TimestampVisitor {
    type Value = Timestamp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an ISO8601 UTC datetime")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match DateTime::parse_from_rfc3339(v) {
            Ok(result) => Ok(Timestamp(result.to_utc())),
            Err(e) => Err(E::custom(e)),
        }
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TimestampVisitor {})
    }
}
