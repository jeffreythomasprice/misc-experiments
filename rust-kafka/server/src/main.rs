mod kafka;
mod websockets;

use std::{env, net::SocketAddr};

use anyhow::Result;
use axum::{
    extract::{ws::WebSocket, ConnectInfo, FromRef, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get},
    serve, Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use clap::{command, Parser, Subcommand};
use kafka::{consume, produce, ConsumerConfig, Message, ProducerConfig};
use rdkafka::util::get_rdkafka_version;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// #[derive(Parser)]
// #[command(version, about, long_about = None)]
// struct Cli {
//     #[arg(long)]
//     bootstrap_servers: String,

//     #[command(subcommand)]
//     command: Commands,
// }

// #[derive(Subcommand)]
// enum Commands {
//     Consumer {
//         #[arg(short, long)]
//         topics: Vec<String>,
//     },
//     Producer {
//         #[arg(short, long)]
//         topic: String,

//         #[arg(short, long)]
//         message: String,
//     },
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct MessagePayload {
//     message: String,
// }

#[derive(Clone)]
struct AppState {
    websockets: websockets::ConnectedClients,
}

impl FromRef<AppState> for websockets::ConnectedClients {
    fn from_ref(input: &AppState) -> Self {
        input.websockets.clone()
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
    };

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/ws", any(websockets::handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)));
    let listener = TcpListener::bind("127.0.0.1:8001").await?;
    serve(listener, app).await?;

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

async fn hello_world() -> (StatusCode, String) {
    (StatusCode::IM_A_TEAPOT, "I'm a teapot!".to_owned())
}
