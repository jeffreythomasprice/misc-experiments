use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, Pool, Postgres};

#[derive(Debug, Clone)]
pub struct Message {
    pub id: i32,
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

    pub async fn get_by_id(&self, id: i32) -> Result<Option<Message>> {
        match query_as!(Message, r"SELECT * FROM messages WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(result) => Ok(Some(result)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(anyhow!("error getting message by id: {:?}", e))?,
        }
    }

    pub async fn insert(&self, message: Create) -> Result<()> {
        query!(
            r"INSERT INTO messages (timestamp, sender, message) VALUES ($1, $2, $3)",
            message.timestamp,
            message.sender,
            message.message
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("error inserting new message: {e:?}"))?;
        Ok(())
    }
}
