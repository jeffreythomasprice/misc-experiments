mod kafka;

use std::env;

use anyhow::Result;
use clap::{command, Parser, Subcommand};
use kafka::{consume, produce, ConsumerConfig, Message, ProducerConfig};
use rdkafka::util::get_rdkafka_version;
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
struct MessagePayload {
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    match env::var(EnvFilter::DEFAULT_ENV).as_deref() {
        Ok("") | Err(_) => {
            // TODO find the name of this project dynamically, don't just hard-code "server"
            env::set_var(EnvFilter::DEFAULT_ENV, "server=TRACE");
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
            let mut receiver = consume::<MessagePayload>(ConsumerConfig {
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
            let sender = produce(ProducerConfig {
                bootstrap_servers: cli.bootstrap_servers,
            })
            .await?;
            sender
                .send(Message {
                    topic,
                    key: Uuid::new_v7(Timestamp::now(ContextV7::new())).to_string(),
                    headers: [(
                        "example header".to_owned(),
                        "example header value".to_owned(),
                    )]
                    .into_iter()
                    .collect(),
                    payload: MessagePayload { message },
                })
                .await?;
            Ok(())
        }
    }
}
