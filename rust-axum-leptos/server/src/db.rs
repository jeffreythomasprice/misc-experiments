use std::str::FromStr;

use sqlx::{
    query, query_as, sqlite::SqliteConnectOptions, Connection, Executor, FromRow, Row, SqlitePool,
};
use tracing::*;

pub struct Service {
    pool: SqlitePool,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub username: String,
}

impl Service {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite://sqlite.db")?.create_if_missing(true),
        )
        .await?;

        let mut conn = pool.acquire().await?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                is_admin BOOLEAN NOT NULL
            );
            ",
        )
        .await?;

        conn.transaction(|t| {
            Box::pin(async move {
                let count: i64 = query("SELECT count(*) FROM users WHERE username = ?")
                    .bind("admin")
                    .fetch_one(&mut **t)
                    .await?
                    .get(0);
                if count == 0 {
                    info!("creating default user");
                    query("INSERT INTO users (username, password, is_admin) VALUES (?, ?, ?)")
                        .bind("admin")
                        .bind("admin")
                        .bind(true)
                        .execute(&mut **t)
                        .await?;
                }
                Ok::<(), sqlx::Error>(())
            })
        })
        .await?;

        Ok(Self { pool })
    }

    pub async fn check_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>("SELECT username FROM users WHERE username = ? AND password = ?")
            .bind(username)
            .bind(password)
            .fetch_optional(&self.pool)
            .await
    }
}