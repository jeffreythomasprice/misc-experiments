use std::net::SocketAddr;
use std::str::FromStr;

use axum::{routing::*, Json};
use shared::models::messages::{ClientHelloRequest, GenericResponse};
use tower::ServiceBuilder;
use tower_http::{cors, trace::TraceLayer};
use tracing::*;
use tracing_subscriber::prelude::*;

async fn client_hello(request: Json<ClientHelloRequest>) -> Json<GenericResponse> {
    info!("request = {request:?}");
    // TODO JEFF handle client hello, assign id and store in state?
    Json(GenericResponse::ok())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_str(
                vec![
                    // defaults first
                    "info".to_string(),
                    "tower_http=debug".to_string(),
                    "server=trace".to_string(),
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

    let app = Router::new().route("/client", post(client_hello)).layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(cors),
    );

    axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8001").unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
