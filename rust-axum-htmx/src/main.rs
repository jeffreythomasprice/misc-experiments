mod cache;
mod templates;

use std::{
    collections::HashMap,
    fmt::Debug,
    net::SocketAddr,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use anyhow::anyhow;
use axum::{
    extract::{
        ws::{self, WebSocket},
        ConnectInfo, FromRef, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{any, get, post},
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use envconfig::Envconfig;
use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};
use mustache::Template;
use serde::Serialize;
use templates::Templates;
use tokio::{spawn, sync::Mutex, task::spawn_local};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "HOST")]
    pub host: String,

    #[envconfig(from = "PORT")]
    pub port: u16,
}

#[derive(Debug, Clone)]
struct HttpError {
    status: StatusCode,
    message: String,
}

impl From<anyhow::Error> for HttpError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("{value:}"),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}

#[derive(Clone)]
struct ActiveWebsocketConnection {
    id: Uuid,
    addr: SocketAddr,
    sink: Arc<Mutex<SplitSink<WebSocket, ws::Message>>>,
}

impl ActiveWebsocketConnection {
    fn new<IncomingMessageCallback, CloseCallback>(
        socket: WebSocket,
        addr: SocketAddr,
        incoming: IncomingMessageCallback,
        close: CloseCallback,
    ) -> Self
    where
        IncomingMessageCallback: Fn(String) -> anyhow::Result<()> + Send + 'static,
        CloseCallback: Fn(&Self) -> anyhow::Result<()> + Send + 'static,
    {
        let (sink, mut stream) = socket.split();

        let result = Self {
            id: Uuid::new_v4(),
            addr,
            sink: Arc::new(Mutex::new(sink)),
        };

        {
            let result = result.clone();
            spawn(async move {
                if let Some(message) = stream.next().await {
                    match message {
                        Ok(ws::Message::Text(message)) => {
                            trace!(
                                "received websocket message, websocket: {:?}, message: {}",
                                result,
                                message
                            );
                            if let Err(e) = incoming(message) {
                                error!("error in websocket message handler, websocket: {:?}, error: {:?}", result, e);
                            }
                        }
                        Ok(ws::Message::Binary(message)) => {
                            trace!("TODO received websocket binary message: {:?}", message);
                            todo!()
                        }
                        Ok(ws::Message::Close(_)) => {
                            debug!("websocket closed: {:?}", result);
                            if let Err(e) = close(&result) {
                                error!("error in websocket close handler while handling close event, websocket: {:?}, error: {:?}", result,e);
                            }
                        }
                        Ok(ws::Message::Ping(_)) | Ok(ws::Message::Pong(_)) => (),
                        Err(e) => error!("error receiving websocket message: {:?}", e),
                    }
                }
                trace!("websocket loop closed, websocket: {:?}", result);
                if let Err(e) = close(&result) {
                    error!("error in websocket close handler while handling websocket loop closed, websocket: {:?}, error: {:?}", result,e);
                }
            });
        }

        result
    }

    async fn send(&self, s: String) -> anyhow::Result<()> {
        let sink = &mut *self.sink.lock().await;
        trace!("sending to {:?}, message={}", self, s);
        sink.send(ws::Message::Text(s)).await?;
        Ok(())
    }
}

impl Debug for ActiveWebsocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveWebsocketConnection")
            .field("id", &self.id)
            .field("addr", &self.addr)
            .finish()
    }
}

#[derive(Clone)]
struct AppState {
    templates: Templates,
    clicks: Arc<Mutex<u64>>,
    websockets: Arc<Mutex<HashMap<Uuid, ActiveWebsocketConnection>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            templates: Templates::new(),
            clicks: Arc::new(Mutex::new(0)),
            websockets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn new_websocket_connection(&mut self, ws: ActiveWebsocketConnection) {
        let websockets = &mut *self.websockets.lock().await;
        info!("registered new active websocket connection: {:?}", ws);
        websockets.insert(ws.id, ws);
    }

    async fn close_websocket_connection(&mut self, ws: ActiveWebsocketConnection) {
        info!("removing websocket connection: {:?}", ws);
        let websockets = &mut *self.websockets.lock().await;
        websockets.remove(&ws.id);
    }

    async fn broadcast(&mut self, message: String) -> Result<(), HttpError> {
        let message = self
            .templates
            .template_path_to_string(
                "templates/new-message.html",
                &NewMessage { content: message },
            )
            .await?;
        info!("broadcasting {}", message);
        let websockets = &*self.websockets.lock().await;
        for ws in websockets.values() {
            if let Err(e) = ws.send(message.clone()).await {
                error!(
                    "error sending message to websocket: {:?}, error: {:?}",
                    ws, e
                );
            }
        }
        Ok(())
    }

    async fn get_clicks(&self) -> u64 {
        *self.clicks.lock().await
    }

    async fn click(&mut self) -> u64 {
        let clicks = &mut *self.clicks.lock().await;
        *clicks += 1;
        *clicks
    }
}

impl FromRef<AppState> for Templates {
    fn from_ref(input: &AppState) -> Self {
        input.templates.clone()
    }
}

#[derive(Serialize)]
struct Index {
    content: String,
}

#[derive(Serialize)]
struct Counter {
    clicks: u64,
}

#[derive(Serialize)]
struct NewMessage {
    content: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::from_filename("local.env")?;
    let config = Config::init_from_env().unwrap();

    let state = AppState::new();

    let app = Router::new()
        .nest_service(
            "/static",
            ServeDir::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")),
        )
        .route("/", get(index))
        .route("/websocket", any(websocket))
        .route("/click", post(click))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let serve_result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    );
    info!("listening at {}", addr);
    serve_result.await?;

    Ok(())
}

async fn websocket(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!(
        "websocket connected, addr: {}, user agent: {:?}",
        addr, user_agent
    );
    ws.on_upgrade(move |socket| websocket_upgrade(state, socket, addr))
}

async fn websocket_upgrade(state: AppState, socket: WebSocket, addr: SocketAddr) {
    state
        .clone()
        .new_websocket_connection(ActiveWebsocketConnection::new(
            socket,
            addr,
            {
                let state = state.clone();
                move |message| {
                    let mut state = state.clone();
                    spawn(async move {
                        if let Err(e) = state
                            .broadcast(format!("TODO response to {}", message))
                            .await
                        {
                            error!("error broadcasting: {:?}", e);
                        }
                    });
                    Ok(())
                }
            },
            {
                let state = state.clone();
                move |ws| {
                    let mut state = state.clone();
                    let ws = ws.clone();
                    spawn(async move {
                        state.close_websocket_connection(ws).await;
                    });
                    Ok(())
                }
            },
        ))
        .await
}

async fn index(
    State(state): State<AppState>,
    State(mut templates): State<Templates>,
) -> Result<impl IntoResponse, HttpError> {
    let counter = templates
        .template_path_to_string(
            "templates/counter.html",
            &Counter {
                clicks: state.get_clicks().await,
            },
        )
        .await?;
    let messages = templates
        .template_path_to_string("templates/messages.html", &0)
        .await?;
    let content = counter + &messages;
    Ok(Html(
        templates
            .template_path_to_string("templates/index.html", &Index { content })
            .await?,
    ))
}

async fn click(
    State(mut state): State<AppState>,
    State(mut templates): State<Templates>,
) -> Result<impl IntoResponse, HttpError> {
    let clicks = state.click().await;
    info!("click, new counter: {}", clicks);
    Ok(Html(
        templates
            .template_path_to_string("templates/counter.html", &Counter { clicks })
            .await?,
    ))
}
