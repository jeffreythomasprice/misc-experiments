use std::{collections::HashMap, fmt::Debug, time::Duration};

use anyhow::{anyhow, Result};
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::RDKafkaLogLevel,
    consumer::{Consumer, ConsumerContext, StreamConsumer},
    message::{Header, Headers, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    types::RDKafkaErrorCode,
    ClientConfig, ClientContext, Message as _,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    spawn,
    sync::mpsc::{channel, Receiver, Sender},
    task::spawn_local,
};
use tracing::*;

#[derive(Debug)]
pub struct ConsumerConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub topics: Vec<String>,
}

#[derive(Debug)]
pub struct ProducerConfig {
    pub bootstrap_servers: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub topic: String,
    pub key: String,
    pub headers: HashMap<String, String>,
    pub payload: T,
}

struct KafkaContext;

impl ClientContext for KafkaContext {
    fn log(&self, level: RDKafkaLogLevel, fac: &str, log_message: &str) {
        match level {
            RDKafkaLogLevel::Emerg
            | RDKafkaLogLevel::Alert
            | RDKafkaLogLevel::Critical
            | RDKafkaLogLevel::Error => error!(
                "kafka log, level: {:?}, fac: {}, message: {}",
                level, fac, log_message
            ),
            RDKafkaLogLevel::Warning => warn!(
                "kafka log, level: {:?}, fac: {}, message: {}",
                level, fac, log_message
            ),
            RDKafkaLogLevel::Notice | RDKafkaLogLevel::Info => info!(
                "kafka log, level: {:?}, fac: {}, message: {}",
                level, fac, log_message
            ),
            RDKafkaLogLevel::Debug => debug!(
                "kafka log, level: {:?}, fac: {}, message: {}",
                level, fac, log_message
            ),
        }
    }

    fn error(&self, error: rdkafka::error::KafkaError, reason: &str) {
        error!("kafka error, err: {:?}, reason: {}", error, reason);
    }
}

impl ConsumerContext for KafkaContext {
    fn pre_rebalance(&self, rebalance: &rdkafka::consumer::Rebalance<'_>) {
        info!("pre_rebalance: {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &rdkafka::consumer::Rebalance<'_>) {
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

pub async fn consume<T>(config: ConsumerConfig) -> Result<Receiver<Message<T>>>
where
    T: DeserializeOwned + Debug + 'static,
{
    if config.topics.is_empty() {
        Err(anyhow!("must provide at least one topic"))?;
    }

    create_topics(&config.bootstrap_servers, &config.topics).await?;

    let context = KafkaContext;

    let consumer: StreamConsumer<KafkaContext> = client_config(&config.bootstrap_servers)
        .set("group.id", config.group_id)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
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
    trace!("consumer subscribed to topics: {:?}", config.topics);

    let (sender, receiver) = channel(1);

    spawn_local(async move {
        loop {
            let message = match consumer.recv().await {
                Ok(message) => message,
                Err(e) => {
                    error!("error receiving message from kafka: {e:?}");
                    continue;
                }
            };

            let key = match message.key().map(|s| std::str::from_utf8(s)) {
                Some(Ok(key)) => key.to_owned(),
                Some(Err(e)) => {
                    warn!(
                        "message key isn't utf8, topic: {}, error: {:?}",
                        message.topic(),
                        e
                    );
                    continue;
                }
                None => {
                    warn!("message missing key, topic: {}", message.topic());
                    continue;
                }
            };

            let headers = match message.headers() {
                Some(headers) => {
                    let mut results = HashMap::new();
                    results.reserve(headers.count());
                    for header in headers.iter() {
                        let value = match header.value.map(|value| std::str::from_utf8(value)) {
                            Some(Ok(value)) => value,
                            Some(Err(e)) => {
                                warn!("header value failed to parse as utf8, topic: {}, key: {}, header: {}, error: {:?}", message.topic(), key, header.key, e);
                                continue;
                            }
                            None => {
                                warn!(
                                    "header value missing, topic: {}, key: {}, header: {}",
                                    message.topic(),
                                    key,
                                    header.key
                                );
                                continue;
                            }
                        };
                        results.insert(header.key.to_owned(), value.to_owned());
                    }
                    results
                }
                None => HashMap::new(),
            };

            let payload = match message.payload().map(|s| std::str::from_utf8(s)) {
                Some(Ok(payload)) => payload,
                Some(Err(e)) => {
                    warn!(
                        "message payload isn't utf8, topic: {}, key: {}, error: {:?}",
                        message.topic(),
                        key,
                        e
                    );
                    continue;
                }
                None => {
                    warn!(
                        "message missing payload, topic: {}, key: {}",
                        message.topic(),
                        key
                    );
                    continue;
                }
            };

            let payload = match serde_json::from_str::<T>(payload) {
                Ok(payload) => payload,
                Err(e) => {
                    warn!("error deserializing message payload as expected type, topic: {}, key: {}, payload: {}, error: {:?}", message.topic(), key, payload, e);
                    continue;
                }
            };

            let parsed_message = Message {
                topic: message.topic().to_owned(),
                key,
                headers,
                payload,
            };
            trace!("received message: {:?}", parsed_message);
            let key = parsed_message.key.clone();
            if let Err(e) = sender.send(parsed_message).await {
                error!(
                    "error sending parsed message to channel, topic: {}, key: {}, error: {:?}",
                    message.topic(),
                    key,
                    e
                );
            }

            if let Err(e) = consumer.commit_message(&message, rdkafka::consumer::CommitMode::Async)
            {
                error!("error committing message to kafka: {e:?}");
            }
        }
    });

    Ok(receiver)
}

pub async fn produce<T>(config: ProducerConfig) -> Result<Sender<Message<T>>>
where
    T: Serialize + Debug + 'static,
{
    debug!("creating producer: {:?}", config);

    let producer: FutureProducer = client_config(&config.bootstrap_servers)
        .create()
        .map_err(|e| anyhow!("error creating producer: {e:?}"))?;

    let (sender, mut receiver) = channel::<Message<T>>(1);

    spawn_local(async move {
        while let Some(message) = receiver.recv().await {
            info!("sending message: {:?}", message);

            let mut headers = OwnedHeaders::new();
            for (key, value) in message.headers.iter() {
                headers = headers.insert(Header {
                    key,
                    value: Some(&value),
                });
            }

            let payload = match serde_json::to_string(&message.payload) {
                Ok(payload) => payload,
                Err(e) => {
                    error!(
                        "error converting message to json, message: {:?}, error: {:?}",
                        message, e
                    );
                    continue;
                }
            };

            match producer
                .send(
                    FutureRecord::to(&message.topic)
                        .payload(&payload)
                        .key(message.key.as_bytes())
                        .headers(headers),
                    Duration::from_secs(0),
                )
                .await
            {
                Ok(result) => debug!("message sent, message: {:?}, result: {:?}", message, result),
                Err(e) => error!("error sending message: {e:?}"),
            };
        }
    });

    Ok(sender)
}

async fn create_topics(bootstrap_servers: &str, topics: &Vec<String>) -> Result<()> {
    info!("creating topics: {:?}", topics);

    let client: AdminClient<DefaultClientContext> = client_config(bootstrap_servers).create()?;

    let topics = topics
        .iter()
        .map(|topic| NewTopic {
            name: topic,
            num_partitions: 3,
            replication: TopicReplication::Fixed(1),
            config: vec![],
        })
        .collect::<Vec<_>>();
    debug!("topic configs: {:?}", topics);

    let results = client
        .create_topics(topics.iter().collect::<Vec<_>>(), &AdminOptions::default())
        .await
        .map_err(|e| anyhow!("error creating topics: {:?}, error: {:?}", topics, e))?;
    for result in results {
        match result {
            Ok(topic) => trace!("topic created: {topic}"),
            Err((topic, RDKafkaErrorCode::TopicAlreadyExists)) => {
                trace!("topic already exists: {topic}")
            }
            Err((topic, e)) => Err(anyhow!("error creating topic {}, {:?}", topic, e))?,
        };
    }

    Ok(())
}

fn client_config(bootstrap_servers: &str) -> ClientConfig {
    // https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md
    let mut result = ClientConfig::new();
    result
        .set("security.protocol", "plaintext")
        .set("bootstrap.servers", bootstrap_servers)
        .set_log_level(RDKafkaLogLevel::Debug);
    result
}