use anyhow::Result;
use axum::{extract::State, http::StatusCode, Json};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use shared::{Id, Timestamp};
use tokio::spawn;
use tracing::*;

use crate::{
    services::{
        self,
        kafka::{consume, list_topics, produce, ConsumerConfig, ProducerConfig},
    },
    Config,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Id,
    pub timestamp: Timestamp,
    pub sender: Id,
    pub payload: String,
}

#[derive(Clone)]
pub struct Kafka {
    bootstrap_servers: String,
    all_messages_topic_name: String,
    channel_topic_name_prefix: String,
}

impl Kafka {
    pub async fn new(config: &Config) -> Result<Self> {
        let result = Self {
            bootstrap_servers: config.bootstrap_servers.clone(),
            all_messages_topic_name: config.all_messages_topic_name.clone(),
            channel_topic_name_prefix: config.channel_topic_name_prefix.clone(),
        };

        let mut receiver = consume::<Message>(ConsumerConfig {
            bootstrap_servers: config.bootstrap_servers.clone(),
            group_id: config.all_messages_consumer_group_id.clone(),
            topics: vec![config.all_messages_topic_name.clone()],
        })
        .await?;

        spawn(async move {
            while let Some(message) = receiver.recv().await {
                info!("TODO received message over kafka all messages topic: {:?}", message);
            }
        });

        Ok(result)
    }

    pub async fn get_channels(&self) -> Result<Vec<String>> {
        let topics = list_topics(&self.bootstrap_servers).await?;
        Ok(topics
            .into_iter()
            .filter(|topic| topic.starts_with(&self.channel_topic_name_prefix))
            .collect())
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        let sender = produce(ProducerConfig {
            bootstrap_servers: self.bootstrap_servers.clone(),
        })
        .await?;

        sender
            .send(services::kafka::Message {
                topic: self.all_messages_topic_name.clone(),
                key: message.id.to_string(),
                headers: None,
                payload: message,
            })
            .await?;

        Ok(())
    }

    /*
    TODO more server functionality
    all topic consumer should re-write to destination-specific topics
    websockets should request lists of topics they care about and the websoccket handlers should update a list of topics to listen to
    */
}

pub async fn get_channels(State(kafka): State<Kafka>) -> Result<Json<Vec<String>>, StatusCode> {
    match kafka.get_channels().await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("error getting channels: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
