mod app;
mod services;

use std::{env, fs::File, net::SocketAddr};

use anyhow::Result;
use app::{app::AppState, kafka::Kafka, websockets::ConnectedClients};
use axum::{
    routing::{any, get},
    serve, Router,
};
use clap::Parser;
use rdkafka::util::get_rdkafka_version;
use serde::Deserialize;
use services::kafka::list_topics;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
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
    channel_topic_name_prefix: String,
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
        websockets: ConnectedClients::new(),
        kafka: Kafka::new(&config).await?,
    };

    let app = Router::new()
        .route("/channels", get(app::kafka::get_channels))
        .route("/ws", any(app::websockets::handler))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods(tower_http::cors::Any)
                .allow_origin(tower_http::cors::Any),
        )
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
