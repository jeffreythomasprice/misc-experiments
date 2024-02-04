mod db;
mod websockets;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        ConnectInfo, FromRef, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures_util::StreamExt;
use shared::{
    LoginRequest, LoginResponse, WebSocketClientToServerMessage, WebSocketServerToClientMessage,
};
use tokio::{
    spawn,
    task::{spawn_local, LocalSet},
};
use tower_http::{
    cors,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type WebSocketsService = websockets::Service<WebSocketServerToClientMessage>;

#[derive(Clone)]
struct AppState {
    db: Arc<db::Service>,
    websockets: Arc<WebSocketsService>,
}

impl FromRef<AppState> for Arc<db::Service> {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppState> for Arc<WebSocketsService> {
    fn from_ref(input: &AppState) -> Self {
        input.websockets.clone()
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
    let db = db::Service::new().await.unwrap();

    let app = Router::new()
        .route("/login", post(login_handler))
        .route("/ws", get(websocket_handler))
        .with_state(AppState {
            db: Arc::new(db),
            websockets: Arc::new(WebSocketsService::new()),
        })
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // let local_set = LocalSet::new();
    // local_set
    //     .run_until(async move {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    // })
    // .await;
}

async fn login_handler(
    State(db): State<Arc<db::Service>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Error> {
    debug!("login request: {request:?}");
    if let Some(user) = db
        .check_password(&request.username, &request.password)
        .await?
    {
        debug!("auth successful: {user:?}");
        Ok(Json(LoginResponse {
            username: user.username,
        }))
    } else {
        debug!("auth failed");
        Err(Error::Unauthorized)
    }
}

async fn websocket_handler(
    State(service): State<Arc<WebSocketsService>>,
    ws: WebSocketUpgrade,
    // user_agent: Option<TypedHeader<UserAgent>>,
    // ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    // let user_agent = user_agent
    //     .map(|value| value.to_string())
    //     .unwrap_or_else(|| "<failed to parse header>".to_string());
    // debug!("new websocket connection, user agent = {user_agent}, addr = {addr}");

    async fn f(service: Arc<WebSocketsService>, socket: axum::extract::ws::WebSocket) {
        let (send, mut receive) = service
            .client_connected::<WebSocketClientToServerMessage>(socket)
            .await;

        spawn(async move {
            if let Err(e) = send.send(WebSocketServerToClientMessage::Placeholder(
                "TODO hello from server".to_string(),
            )) {
                error!("error sending: {e:?}");
            }
        });

        // spawn(async move {
        let mut done = false;
        while !done {
            match receive.recv().await {
                Ok(message) => {
                    debug!("TODO received: {message:?}");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    done = true;
                }
                Err(e) => {
                    error!("error receiving message: {e:?}");
                }
            }
        }
        debug!("TODO websocket closed");
        // });
    }

    ws.on_upgrade(move |socket| f(service, socket))
}
