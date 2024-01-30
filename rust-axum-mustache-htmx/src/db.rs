use std::str::FromStr;

use sqlx::{
    migrate::MigrateDatabase,
    query,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Connection, Executor, Row, Sqlite, SqliteConnection, SqlitePool,
};
use tracing::*;

pub struct DbService {
    pool: SqlitePool,
}

impl DbService {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite://target/sqlite.db")?.create_if_missing(true),
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
    ) -> Result<bool, sqlx::Error> {
        let results: i32 = query("SELECT count(*) FROM users WHERE username = ? AND password = ?")
            .bind(username)
            .bind(password)
            .fetch_one(&self.pool)
            .await?
            .get(0);
        Ok(results == 1)
    }
}
