mod db;

use std::sync::Arc;

use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rusqlite::Connection;
use serde::Serialize;
use shared::{LoginRequest, LoginResponse};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct ErrorResponse {
    pub status_code: StatusCode,
    pub messages: Vec<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(shared::ErrorResponse {
                messages: self.messages.clone(),
            }),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            messages: vec![format!("{value:?}")],
        }
    }
}

#[derive(Clone)]
struct AppState {
    db_conn: Arc<Mutex<Connection>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                // TODO turn down axum stuff?
                .parse_lossy("TODO some filter here"),
            // "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
        )
        .init();

    let db_path = "./db.sqlite";
    info!("db: {db_path}");
    let db_conn = Connection::open(db_path)?;
    db::run_migrations(&db_conn)?;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app_state = AppState {
        db_conn: Arc::new(Mutex::new(db_conn)),
    };

    let app = Router::new()
        .route("/login", post(login))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    let server_addr = "127.0.0.1:8001";
    let listener = tokio::net::TcpListener::bind(server_addr).await?;
    info!("listening at {server_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn login(
    State(state): State<AppState>,
    request: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ErrorResponse> {
    let db_conn = state.db_conn.lock().await;
    info!("TODO request = {request:?}");
    info!(
        "TODO pw check? {:?}",
        db::check_password(&db_conn, "admin", "admin")?
    );
    info!(
        "TODO pw check? {:?}",
        db::check_password(&db_conn, "admin", "admin2")?
    );

    Err(ErrorResponse {
        status_code: StatusCode::UNAUTHORIZED,
        messages: vec!["TODO not implemented".into()],
    })
}
