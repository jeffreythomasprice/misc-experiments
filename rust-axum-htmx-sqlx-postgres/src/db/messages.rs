use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Create {
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub message: String,
}

#[derive(Clone)]
pub struct Dao {
    pool: Pool<Postgres>,
}

impl Dao {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: u64) -> Option<Message> {
        todo!()
    }

    pub async fn insert(&self, message: Create) {
        todo!()
    }
}
