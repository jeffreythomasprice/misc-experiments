use std::{str::FromStr, time::SystemTime};

use chrono::Utc;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Id(uuid::Uuid);

impl Id {
    pub fn new() -> Id {
        Id(uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::ContextV7::new())))
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct IdVisitor {}

impl<'de> Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a uuid")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match uuid::Uuid::from_str(v) {
            Ok(result) => Ok(Id(result)),
            Err(e) => Err(E::custom(e)),
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IdVisitor {})
    }
}
