mod kafka;

use std::{
    env,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Result};
use clap::{command, Parser, Subcommand};
use kafka::{consume, produce, ConsumerConfig, CustomMessage, ProducerConfig};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::{Header, Headers, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    util::get_rdkafka_version,
};
use serde::{Deserialize, Serialize};
use tracing::*;
use tracing_subscriber::EnvFilter;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    bootstrap_servers: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Consumer {
        #[arg(short, long)]
        topics: Vec<String>,
    },
    Producer {
        #[arg(short, long)]
        topic: String,

        #[arg(short, long)]
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    match env::var(EnvFilter::DEFAULT_ENV).as_deref() {
        Ok("") | Err(_) => {
            env::set_var(EnvFilter::DEFAULT_ENV, "rust_kafka=TRACE");
        }
        _ => (),
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("rdkafka version = {:?}", get_rdkafka_version());

    let cli = Cli::parse();
    match cli.command {
        Commands::Consumer { topics } => {
            let mut receiver = consume::<Message>(ConsumerConfig {
                bootstrap_servers: cli.bootstrap_servers,
                group_id: "group id".to_owned(),
                topics,
            })
            .await?;
            while let Some(message) = receiver.recv().await {
                info!("received message: {:?}", message);
            }
            Ok(())
        }

        Commands::Producer { topic, message } => {
            produce(ProducerConfig {
                bootstrap_servers: cli.bootstrap_servers,
                message: CustomMessage {
                    topic,
                    key: Uuid::new_v7(Timestamp::now(ContextV7::new())).to_string(),
                    headers: [(
                        "example header".to_owned(),
                        "example header value".to_owned(),
                    )]
                    .into_iter()
                    .collect(),
                    payload: Message { message },
                },
            })
            .await
        }
    }
}
