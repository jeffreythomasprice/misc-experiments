use anyhow::Result;
use envconfig::Envconfig;
use serde;
use serde::{de::DeserializeOwned, Deserialize};
use sqlx::{
    migrate,
    postgres::{PgListener, PgNotification},
    query, query_file, Pool, Postgres,
};
use std::{fmt::Debug, ops::Not};
use tokio::spawn;
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,

    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,

    #[envconfig(from = "POSTGRES_HOST")]
    pub postgres_host: String,

    #[envconfig(from = "POSTGRES_PORT")]
    pub postgres_port: u16,

    #[envconfig(from = "POSTGRES_DB")]
    pub postgres_db: String,
}

#[derive(Deserialize, Debug)]
pub struct Notification {
    pub id: u64,
    pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum PayloadVariant {
    #[serde(rename = "INSERT")]
    Insert { new: Notification },
    #[serde(rename = "UPDATE")]
    Update {
        old: Notification,
        new: Notification,
    },
    #[serde(rename = "DELETE")]
    Delete { old: Notification },
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub table: String,
    #[serde(flatten)]
    pub variant: PayloadVariant,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::from_filename("local.env")?;
    let config = Config::init_from_env().unwrap();

    debug!("postgres_user = {}", config.postgres_user);
    // debug!("postgres_password = {}", config.postgres_password);
    debug!("postgres_host = {}", config.postgres_host);
    debug!("postgres_port = {}", config.postgres_port);
    debug!("postgres_db = {}", config.postgres_db);

    let pool = sqlx::PgPool::connect(
        format!(
            "postgres://{}:{}@{}:{}/{}",
            config.postgres_user,
            config.postgres_password,
            config.postgres_host,
            config.postgres_port,
            config.postgres_db
        )
        .as_str(),
    )
    .await?;

    sqlx::migrate!().run(&pool).await?;

    let listener = spawn(async move {
        if let Err(e) = listen(&pool, &vec!["table_update"], |notification: Payload| {
            trace!("notification: {notification:?}");
            Ok(())
        })
        .await
        {
            panic!("listen failed: {e:?}");
        }
    });

    listener.await?;

    Ok(())
}

async fn listen<T, F>(pool: &Pool<Postgres>, channels: &[&str], f: F) -> Result<()>
where
    T: DeserializeOwned + Debug,
    F: Fn(T) -> Result<()>,
{
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen_all(channels.iter().map(|s| *s)).await?;
    loop {
        while let Some(notification) = listener.try_recv().await? {
            if let Err(e) = (|| -> Result<()> {
                trace!(
                    "notification on channel = {}, notification = {}",
                    notification.channel(),
                    notification.payload(),
                );
                f(serde_json::from_str(notification.payload())?)
            })() {
                error!("error handling notification: {e:?}");
            }
        }
        debug!("connection lost, trying again");
    }
}
