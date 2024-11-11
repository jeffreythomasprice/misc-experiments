mod kafka;
mod websockets;

use std::{env, net::SocketAddr};

use anyhow::Result;
use axum::{
    extract::FromRef,
    routing::any,
    serve, Router,
};
use kafka::{consume, produce, ConsumerConfig, ProducerConfig};
use rdkafka::util::get_rdkafka_version;
use serde::{Deserialize, Serialize};
use shared::{Id, Timestamp};
use tokio::{net::TcpListener, spawn};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// TODO use a config file
const BOOTSTRAP_SERVERS: &str = "localhost:9092";
const ALL_MESSAGES_TOPIC_NAME: &str = "all-messages";
const ALL_MESSAGES_CONSUMER_GROUP_ID: &str = "all-messages";

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    id: Id,
    timestamp: Timestamp,
    sender: Id,
    payload: String,
}

#[derive(Clone)]
struct Kafka {
    bootstrap_servers: String,
}

impl Kafka {
    pub fn new(bootstrap_servers: String) -> Self {
        spawn(async move {
            if let Err(e) = Self::consume_all_messages(BOOTSTRAP_SERVERS.to_string()).await {
                error!("all messages consumer error: {:?}", e);
            }
        });

        Self { bootstrap_servers }
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        let sender = produce(ProducerConfig {
            bootstrap_servers: self.bootstrap_servers.clone(),
        })
        .await?;

        sender
            .send(kafka::Message {
                topic: ALL_MESSAGES_TOPIC_NAME.to_owned(),
                key: message.id.to_string(),
                headers: None,
                payload: message,
            })
            .await?;

        Ok(())
    }

    async fn consume_all_messages(bootstrap_servers: String) -> Result<()> {
        let mut receiver = consume::<Message>(ConsumerConfig {
            bootstrap_servers,
            group_id: ALL_MESSAGES_CONSUMER_GROUP_ID.to_owned(),
            topics: vec![ALL_MESSAGES_TOPIC_NAME.to_owned()],
        })
        .await?;

        while let Some(message) = receiver.recv().await {
            info!("TODO received message over kafka all messages topic: {:?}", message);
        }

        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    websockets: websockets::ConnectedClients,
    kafka: Kafka,
}

impl FromRef<AppState> for Kafka {
    fn from_ref(input: &AppState) -> Self {
        input.kafka.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("rdkafka version = {:?}", get_rdkafka_version());

    let state = AppState {
        websockets: websockets::ConnectedClients::new(),
        kafka: Kafka::new(BOOTSTRAP_SERVERS.to_string()),
    };

    let app = Router::new()
        .route("/ws", any(websockets::handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)));
    let listener = TcpListener::bind("127.0.0.1:8001").await?;
    serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())

    // TODO put kafka stuff back

    // let cli = Cli::parse();
    // match cli.command {
    //     Commands::Consumer { topics } => {
    //         let mut receiver = consume::<MessagePayload>(ConsumerConfig {
    //             bootstrap_servers: cli.bootstrap_servers,
    //             group_id: "group id".to_owned(),
    //             topics,
    //         })
    //         .await?;
    //         while let Some(message) = receiver.recv().await {
    //             info!("received message: {:?}", message);
    //         }
    //         Ok(())
    //     }

    //     Commands::Producer { topic, message } => {
    //         let sender = produce(ProducerConfig {
    //             bootstrap_servers: cli.bootstrap_servers,
    //         })
    //         .await?;
    //         sender
    //             .send(Message {
    //                 topic,
    //                 key: Uuid::new_v7(Timestamp::now(ContextV7::new())).to_string(),
    //                 headers: [(
    //                     "example header".to_owned(),
    //                     "example header value".to_owned(),
    //                 )]
    //                 .into_iter()
    //                 .collect(),
    //                 payload: MessagePayload { message },
    //             })
    //             .await?;
    //         Ok(())
    //     }
    // }
}
