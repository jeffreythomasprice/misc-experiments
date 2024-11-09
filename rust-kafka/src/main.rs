use std::{default, env, time::Duration};

use anyhow::{anyhow, Result};
use clap::{command, Parser, Subcommand};
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::RDKafkaLogLevel,
    consumer::{Consumer, ConsumerContext, StreamConsumer},
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    util::get_rdkafka_version,
    ClientConfig, ClientContext, Message,
};
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

struct ConsumerConfig {
    bootstrap_servers: String,
    group_id: String,
    topics: Vec<String>,
}

struct ProducerConfig {
    bootstrap_servers: String,
    topic: String,
    message: String,
}

struct KafkaContext;

impl ClientContext for KafkaContext {}

impl ConsumerContext for KafkaContext {
    fn pre_rebalance<'a>(&self, rebalance: &rdkafka::consumer::Rebalance<'a>) {
        info!("pre_rebalance: {:?}", rebalance);
    }

    fn post_rebalance<'a>(&self, rebalance: &rdkafka::consumer::Rebalance<'a>) {
        info!("post_rebalance: {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        result: rdkafka::error::KafkaResult<()>,
        offsets: &rdkafka::TopicPartitionList,
    ) {
        info!("commit_callbackcommit_: {:?}, {:?}", result, offsets);
    }
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
            consumer(ConsumerConfig {
                bootstrap_servers: cli.bootstrap_servers,
                group_id: "group id".to_owned(),
                topics,
            })
            .await
        }
        Commands::Producer { topic, message } => {
            producer(ProducerConfig {
                bootstrap_servers: cli.bootstrap_servers,
                topic,
                message,
            })
            .await
        }
    }
}

// TODO should take bootstrap servers
fn client_config() -> ClientConfig {
    // https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md
    let mut result = ClientConfig::new();
    result
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "plaintext")
        .set_log_level(RDKafkaLogLevel::Debug);
    result
}

async fn create_topics(bootstrap_servers: String, topics: &Vec<String>) -> Result<()> {
    info!("creating topics: {:?}", topics);

    let client: AdminClient<DefaultClientContext> = client_config()
        .set("bootstrap.servers", bootstrap_servers)
        .create()?;

    let topics = topics
        .iter()
        .map(|topic| NewTopic {
            name: topic,
            num_partitions: 3,
            replication: TopicReplication::Fixed(2),
            config: vec![],
        })
        .collect::<Vec<_>>();
    debug!("topic configs: {:?}", topics);

    client
        .create_topics(topics.iter().collect::<Vec<_>>(), &AdminOptions::default())
        .await?;

    Ok(())
}

async fn consumer(config: ConsumerConfig) -> Result<()> {
    if config.topics.len() == 0 {
        Err(anyhow!("must provide at least one topic"))?;
    }

    create_topics(config.bootstrap_servers, &config.topics).await?;

    let context = KafkaContext;

    let consumer: StreamConsumer<KafkaContext> =
        { client_config().set("group.id", config.group_id) }
            .create_with_context(context)
            .map_err(|e| anyhow!("error creating consumer: {e:?}"))?;

    consumer.subscribe(
        config
            .topics
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;

    loop {
        let message = consumer
            .recv()
            .await
            .map_err(|e| anyhow!("error receiving message from kafka: {e:?}"))?;
        /*
        TODO better consumer
        let payload = match m.payload_view::<str>() {
                        None => "",
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            warn!("Error while deserializing message payload: {:?}", e);
                            ""
                        }
                    };
                    info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                          m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                    if let Some(headers) = m.headers() {
                        for header in headers.iter() {
                            info!("  Header {:#?}: {:?}", header.key, header.value);
                        }
                    }
        */
        info!("received message: {:?}", message);
        consumer
            .commit_message(&message, rdkafka::consumer::CommitMode::Async)
            .map_err(|e| anyhow!("error committing message to kafka: {e:?}"))?;
    }
}

async fn producer(config: ProducerConfig) -> Result<()> {
    let producer: FutureProducer = client_config()
        .set("bootstrap.servers", config.bootstrap_servers)
        .create()
        .map_err(|e| anyhow!("error creating producer: {e:?}"))?;

    let uuid_context = ContextV7::new();
    let key = Uuid::new_v7(Timestamp::now(uuid_context));

    let result = producer
        .send(
            FutureRecord::to(&config.topic)
                .payload(&config.message)
                .key(key.as_bytes())
                .headers(OwnedHeaders::new().insert(Header {
                    key: "example header key",
                    value: Some("example header value"),
                })),
            Duration::from_secs(0),
        )
        .await
        .map_err(|e| anyhow!("error sending message: {e:?}"))?;
    debug!("message sent: {:?}", result);

    Ok(())
}
