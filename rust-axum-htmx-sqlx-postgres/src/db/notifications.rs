use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};
use sqlx::{postgres::PgListener, Pool, Postgres};
use std::fmt::Debug;
use tracing::*;

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum PayloadVariant {
    #[serde(rename = "INSERT")]
    Insert { new_id: u64 },
    #[serde(rename = "UPDATE")]
    Update { old_id: u64, new_id: u64 },
    #[serde(rename = "DELETE")]
    Delete { old_id: u64 },
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub table: String,
    #[serde(flatten)]
    pub variant: PayloadVariant,
}

pub async fn listen<T, F>(pool: &Pool<Postgres>, channels: &[&str], f: F) -> Result<()>
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
