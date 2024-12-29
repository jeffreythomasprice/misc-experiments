mod concurrent_hashmap;
mod templates;
mod websockets;

use std::{fmt::Debug, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{ws::WebSocket, ConnectInfo, FromRef, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{any, get, post},
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use envconfig::Envconfig;
use serde::Serialize;
use templates::Templates;
use tokio::{spawn, sync::Mutex};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use websockets::{ActiveWebsocketConnection, IncomingWebsocketMessage, WebSockets};

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
struct AppState {
    templates: Templates,
    websockets: WebSockets,
    clicks: Arc<Mutex<u64>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            templates: Templates::new(),
            websockets: WebSockets::new(),
            clicks: Arc::new(Mutex::new(0)),
        }
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

impl FromRef<AppState> for WebSockets {
    fn from_ref(input: &AppState) -> Self {
        input.websockets.clone()
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
    State(websockets): State<WebSockets>,
    State(templates): State<Templates>,
) -> impl IntoResponse {
    async fn f(
        mut websockets: WebSockets,
        templates: Templates,
        socket: WebSocket,
        addr: SocketAddr,
    ) {
        async fn incoming(
            mut websockets: WebSockets,
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

            websockets
                .broadcast(message)
                .await
                .map_err(|e| anyhow!("error responding to websocket message: {:?}", e))?;

            Ok(())
        }

        websockets
            .insert(ActiveWebsocketConnection::new(
                socket,
                addr,
                {
                    let websockets = websockets.clone();
                    move |ws, message| {
                        let websockets = websockets.clone();
                        let templates = templates.clone();
                        let ws = ws.clone();
                        spawn(async move {
                            if let Err(e) = incoming(websockets, templates, ws, message).await {
                                error!("error handling incoming websocket message: {:?}", e);
                            }
                        });
                        Ok(())
                    }
                },
                {
                    let websockets = websockets.clone();
                    move |ws| {
                        let mut websockets = websockets.clone();
                        let ws = ws.clone();
                        spawn(async move {
                            websockets.remove(ws).await;
                        });
                        Ok(())
                    }
                },
            ))
            .await
    }

    info!(
        "websocket connected, addr: {}, user agent: {:?}",
        addr, user_agent
    );
    ws.on_upgrade(move |socket| f(websockets, templates, socket, addr))
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
