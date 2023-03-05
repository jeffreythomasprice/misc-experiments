use sqlx::{Pool, Sqlite};
use std::error::Error;

use super::models::Config;

pub struct Service {
    db: Pool<Sqlite>,
}

impl Service {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Config>, Box<dyn Error>> {
        Ok(sqlx::query_as::<_, Config>(
            "SELECT key, value, created, updated FROM config WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let current_value = self.get(key).await?;
        let now = chrono::Utc::now().naive_utc();
        let result = match current_value {
            Some(_) => sqlx::query("UPDATE config SET value = ?, updated = ? WHERE key = ?")
                .bind(value)
                .bind(now)
                .bind(key),
            None => {
                sqlx::query("INSERT INTO config (key, value, created, updated) VALUES (?, ?, ?, ?)")
                    .bind(key)
                    .bind(value)
                    .bind(now)
                    .bind(now)
            }
        }
        .execute(&self.db)
        .await?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(format!(
                "error updating {}, expected one row to update, got {}",
                key,
                result.rows_affected()
            ))?
        }
    }
}
