mod schema;

use axum::http::{Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use diesel::sqlite::Sqlite;
use diesel::{Connection, SqliteConnection};
use dotenv::dotenv;
use serde::Serialize;
use shared::Example;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                // TODO turn down axum stuff?
                .parse_lossy("TODO some filter here"),
            // "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
        )
        .init();

    let database_url = dotenv::var("DATABASE_URL")?;
    info!("database url = {database_url}");
    let db = SqliteConnection::establish(&database_url)?;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/json", get(json))
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    let server_addr = dotenv::var("SERVER_ADDR")?;
    let listener = tokio::net::TcpListener::bind(server_addr.clone()).await?;
    info!("listening at {server_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> (StatusCode, String) {
    (StatusCode::IM_A_TEAPOT, "I'm a teapot!".into())
}

async fn json() -> Json<Example> {
    Json(Example {
        foo: "Hello, World!".to_owned(),
        bar: 42,
    })
}
