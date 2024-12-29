mod concurrent_hashmap;
mod templates;

use std::{
    collections::HashMap,
    fmt::Debug,
    net::{Incoming, SocketAddr},
    path::PathBuf,
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
use concurrent_hashmap::ConcurrentHashMap;
use envconfig::Envconfig;
use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};
use serde::{Deserialize, Serialize};
use templates::Templates;
use tokio::{spawn, sync::Mutex};
use tower_http::{services::ServeDir, trace::TraceLayer};
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

#[derive(Debug, Deserialize)]
struct IncomingWebsocketMessage {
    pub message: String,
    #[serde(rename = "HEADERS")]
    pub headers: HashMap<String, Option<String>>,
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
        IncomingMessageCallback:
            Fn(&Self, IncomingWebsocketMessage) -> anyhow::Result<()> + Send + 'static,
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
                while let Some(message) = stream.next().await {
                    match message {
                        Ok(ws::Message::Text(message)) => {
                            trace!(
                                "received websocket message, websocket: {:?}, message: {}",
                                result,
                                message
                            );
                            match serde_json::from_str(&message) {
                                Ok(message) => {
                                    if let Err(e) = incoming(&result, message) {
                                        error!("error in websocket message handler, websocket: {:?}, error: {:?}", result, e);
                                    }
                                }
                                Err(e) => error!("error deserializing incoming websocket message, websocket: {:?}, error: {:?}", result, e),
                            };
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
                            return;
                        }
                        Ok(ws::Message::Ping(_)) | Ok(ws::Message::Pong(_)) => (),
                        Err(e) => error!("error receiving websocket message: {:?}", e),
                    }
                }
                trace!("websocket loop closed, websocket: {:?}", result);
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
    websockets: ConcurrentHashMap<Uuid, ActiveWebsocketConnection>,
    clicks: Arc<Mutex<u64>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            templates: Templates::new(),
            websockets: ConcurrentHashMap::new(),
            clicks: Arc::new(Mutex::new(0)),
        }
    }

    async fn new_websocket_connection(&mut self, ws: ActiveWebsocketConnection) {
        info!("registered new active websocket connection: {:?}", ws);
        self.websockets.insert(ws.id, ws).await;
    }

    async fn close_websocket_connection(&mut self, ws: ActiveWebsocketConnection) {
        info!("removing websocket connection: {:?}", ws);
        self.websockets.remove(&ws.id).await;
    }

    async fn broadcast(&mut self, message: String) -> Result<(), HttpError> {
        info!("broadcasting {}", message);
        for ws in self.websockets.values().await {
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
    State(templates): State<Templates>,
) -> impl IntoResponse {
    info!(
        "websocket connected, addr: {}, user agent: {:?}",
        addr, user_agent
    );
    ws.on_upgrade(move |socket| websocket_upgrade(state, templates, socket, addr))
}

async fn websocket_upgrade(
    state: AppState,
    templates: Templates,
    socket: WebSocket,
    addr: SocketAddr,
) {
    async fn incoming(
        mut state: AppState,
        mut templates: Templates,
        ws: ActiveWebsocketConnection,
        message: IncomingWebsocketMessage,
    ) -> anyhow::Result<()> {
        let message = templates
            .template_path_to_string(
                "templates/new-message.html",
                &NewMessage {
                    content: format!("{}: {}", ws.id, message.message),
                },
            )
            .await
            .map_err(|e| {
                anyhow!(
                    "error rendering template to respond to websocket message: {:?}",
                    e
                )
            })?;

        state
            .broadcast(message)
            .await
            .map_err(|e| anyhow!("error responding to websocket message: {:?}", e))?;

        Ok(())
    }

    state
        .clone()
        .new_websocket_connection(ActiveWebsocketConnection::new(
            socket,
            addr,
            {
                let state = state.clone();
                move |ws, message| {
                    let state = state.clone();
                    let templates = templates.clone();
                    let ws = ws.clone();
                    spawn(async move {
                        if let Err(e) = incoming(state, templates, ws, message).await {
                            error!("error handling incoming websocket message: {:?}", e);
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
