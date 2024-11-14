mod kafka;
mod websockets;

use std::{env, fs::File, net::SocketAddr};

use anyhow::Result;
use axum::{extract::FromRef, routing::any, serve, Router};
use clap::Parser;
use kafka::{consume, list_topics, produce, ConsumerConfig, ProducerConfig};
use rdkafka::util::get_rdkafka_version;
use serde::{Deserialize, Serialize};
use shared::{Id, Timestamp};
use tokio::{net::TcpListener, spawn};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short = 'c', long)]
    config_file: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    bootstrap_servers: String,
    all_messages_topic_name: String,
    all_messages_consumer_group_id: String,
}

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
    all_messages_topic_name: String,
}

impl Kafka {
    pub async fn new(config: &Config) -> Result<Self> {
        let result = Self {
            bootstrap_servers: config.bootstrap_servers.clone(),
            all_messages_topic_name: config.all_messages_topic_name.clone(),
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

    pub async fn send_message(&self, message: Message) -> Result<()> {
        let sender = produce(ProducerConfig {
            bootstrap_servers: self.bootstrap_servers.clone(),
        })
        .await?;

        sender
            .send(kafka::Message {
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

    let args = Arguments::parse();
    trace!("args = {:?}", args);
    let config: Config = serde_json::from_reader(File::open(args.config_file)?)?;
    trace!("config = {:?}", config);

    info!("rdkafka version = {:?}", get_rdkafka_version());

    let state = AppState {
        websockets: websockets::ConnectedClients::new(),
        kafka: Kafka::new(&config).await?,
    };

    info!("all kafka topics: {:?}", list_topics(&config.bootstrap_servers).await?);

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
