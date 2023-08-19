#![allow(dead_code)]

use std::str::FromStr;

use std::net::SocketAddr;
use std::time::Duration;

use axum::routing::*;
use axum::{extract::FromRef, middleware};
use tower::ServiceBuilder;
use tower_http::{cors, trace::TraceLayer};
use tracing::*;
use tracing_subscriber::prelude::*;

mod auth;
mod clients;
mod models;
mod websockets;

#[derive(Clone)]
struct AppState {
    auth: auth::Service,
    clients: clients::Service,
}

impl AppState {
    fn new() -> Result<Self, String> {
        Ok(Self {
            auth: auth::Service::new()
                .map_err(|e| format!("failed to make auth service: {e:?}"))?,
            clients: clients::Service::new(),
        })
    }
}

impl FromRef<AppState> for auth::Service {
    fn from_ref(input: &AppState) -> Self {
        input.auth.clone()
    }
}

impl FromRef<AppState> for clients::Service {
    fn from_ref(input: &AppState) -> Self {
        input.clients.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_str(
                [
                    "info".to_string(),
                    "tower_http=debug".to_string(),
                    "server=debug".to_string(),
                    // respect the env var to override
                    match std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV) {
                        Ok(env_value) => env_value,
                        Err(_) => "".to_string(),
                    },
                ]
                .join(",")
                .as_str(),
            )
            .unwrap(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = cors::CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let mut state = AppState::new().unwrap();

    let app = Router::new()
        .nest(
            "/client",
            Router::new()
                // everything above here uses auth, below does not
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth::middleware,
                ))
                .route("/", post(clients::create))
                .route("/ws", get(clients::websocket)),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            state.clients.cleanup();
        }
    });

    let addr = SocketAddr::from_str("127.0.0.1:8001").unwrap();
    let server = axum::Server::bind(&addr).serve(app.into_make_service());
    info!("server started {addr}");
    server.await.unwrap();
}
