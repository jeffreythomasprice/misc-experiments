mod db;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ws::WebSocketUpgrade, ConnectInfo, FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use db::DbService;
use shared::{LoginRequest, LoginResponse};
use tower_http::{
    cors,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    db: Arc<DbService>,
}

impl FromRef<AppState> for Arc<DbService> {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

#[derive(Debug)]
enum Error {
    Sqlx(sqlx::Error),
    Unauthorized,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("response error: {self:?}");
        match self {
            Error::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "".into())
                .add_directive("server=trace".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = cors::CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    // TODO error handling
    let db = DbService::new().await.unwrap();

    let app = Router::new()
        .route("/login", post(login_handler))
        .route("/ws", get(websocket_handler))
        .with_state(AppState { db: Arc::new(db) })
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn login_handler(
    State(db): State<Arc<DbService>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Error> {
    debug!("login request: {request:?}");
    if db
        .check_password(&request.username, &request.password)
        .await?
    {
        debug!("auth successful");
        Ok(Json(LoginResponse {}))
    } else {
        debug!("auth failed");
        Err(Error::Unauthorized)
    }
}

async fn websocket_handler(
    _ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = user_agent
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<failed to parse header>".to_string());
    debug!("new websocket connection, user agent = {user_agent}, addr = {addr}");

    (StatusCode::INTERNAL_SERVER_ERROR, "TODO implement me")
    // ws.on_upgrade(move |socket| handle_socket(socket, addr))
    // https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs
}
