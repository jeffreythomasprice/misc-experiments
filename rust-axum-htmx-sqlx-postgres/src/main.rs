mod concurrent_hashmap;
mod db;
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
use chrono::Utc;
use db::notifications;
use envconfig::Envconfig;
use serde::Serialize;
use sqlx::{Pool, Postgres};
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

    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,

    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,

    #[envconfig(from = "POSTGRES_HOST")]
    pub postgres_host: String,

    #[envconfig(from = "POSTGRES_PORT")]
    pub postgres_port: u16,

    #[envconfig(from = "POSTGRES_DB")]
    pub postgres_db: String,
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
    messages_dao: db::messages::Dao,
    templates: Templates,
    websockets: WebSockets,
    clicks: Arc<Mutex<u64>>,
}

impl AppState {
    fn new(db: Pool<Postgres>) -> Self {
        Self {
            messages_dao: db::messages::Dao::new(db.clone()),
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

impl FromRef<AppState> for db::messages::Dao {
    fn from_ref(input: &AppState) -> Self {
        input.messages_dao.clone()
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
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::from_filename("local.env")?;
    let config = Config::init_from_env().unwrap();
    debug!("host = {}", config.host);
    debug!("port = {}", config.port);
    debug!("postgres_user = {}", config.postgres_user);
    // debug!("postgres_password = {}", config.postgres_password);
    debug!("postgres_host = {}", config.postgres_host);
    debug!("postgres_port = {}", config.postgres_port);
    debug!("postgres_db = {}", config.postgres_db);

    let pool = sqlx::PgPool::connect(
        format!(
            "postgres://{}:{}@{}:{}/{}",
            config.postgres_user, config.postgres_password, config.postgres_host, config.postgres_port, config.postgres_db
        )
        .as_str(),
    )
    .await?;

    sqlx::migrate!().run(&pool).await?;

    let state = AppState::new(pool.clone());

    let postres_listener = {
        let messages_dao = state.messages_dao.clone();
        let templates = state.templates.clone();
        let websockets = state.websockets.clone();
        let last_message_id = Arc::new(Mutex::new(None));
        spawn(async move {
            if let Err(e) = db::notifications::listen(&pool, &vec!["table_update"], {
                |notification: db::notifications::Payload| {
                    debug!("postgres notification: {notification:?}");
                    let messages_dao = messages_dao.clone();
                    let mut templates = templates.clone();
                    let mut websockets = websockets.clone();
                    let last_message_id = last_message_id.clone();
                    spawn(async move {
                        let messages_dao = messages_dao.clone();
                        let last_message_id = &mut *last_message_id.lock().await;
                        match notification.variant {
                            notifications::PayloadVariant::Insert { new_id } => {
                                match last_message_id {
                                    Some(last_message_id) => {
                                        for id in (*last_message_id + 1)..=new_id {
                                            if let Err(e) = broadcast_message(&messages_dao, &mut templates, &mut websockets, id).await {
                                                error!("error broadcasting message id: {}, error: {:?}", id, e);
                                            }
                                        }
                                    }
                                    None => {
                                        if let Err(e) = broadcast_message(&messages_dao, &mut templates, &mut websockets, new_id).await {
                                            error!("error broadcasting message id: {}, error: {:?}", new_id, e);
                                        }
                                    }
                                };
                                *last_message_id = Some(new_id);
                            }
                            // TODO handle message updates and deletes?
                            notifications::PayloadVariant::Update { old_id, new_id } => todo!(),
                            notifications::PayloadVariant::Delete { old_id } => todo!(),
                        };
                    });
                    Ok(())
                }
            })
            .await
            {
                panic!("postgres listen failed: {e:?}");
            }
        })
    };

    let app = Router::new()
        .nest_service("/static", ServeDir::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")))
        .route("/", get(index))
        .route("/websocket", any(websocket))
        .route("/click", post(click))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let serve_result = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());
    info!("listening at {}", addr);

    serve_result.await?;
    postres_listener.await?;

    Ok(())
}

async fn websocket(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(messages_dao): State<db::messages::Dao>,
    State(websockets): State<WebSockets>,
) -> impl IntoResponse {
    async fn incoming(
        messages_dao: db::messages::Dao,
        ws: ActiveWebsocketConnection,
        message: IncomingWebsocketMessage,
    ) -> anyhow::Result<()> {
        messages_dao
            .insert(db::messages::Create {
                timestamp: Utc::now(),
                sender: ws.id.to_string(),
                message: message.message,
            })
            .await;
        Ok(())
    }

    async fn upgrader(messages_dao: db::messages::Dao, mut websockets: WebSockets, socket: WebSocket, addr: SocketAddr) {
        websockets
            .insert(ActiveWebsocketConnection::new(
                socket,
                addr,
                {
                    move |ws, message| {
                        let messages_dao = messages_dao.clone();
                        let ws = ws.clone();
                        spawn(async move {
                            if let Err(e) = incoming(messages_dao, ws, message).await {
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

    info!("websocket connected, addr: {}, user agent: {:?}", addr, user_agent);
    ws.on_upgrade(move |socket| upgrader(messages_dao, websockets, socket, addr))
}

async fn index(State(state): State<AppState>, State(mut templates): State<Templates>) -> Result<impl IntoResponse, HttpError> {
    let counter = templates
        .template_path_to_string(
            "templates/counter.html",
            &Counter {
                clicks: state.get_clicks().await,
            },
        )
        .await?;
    let messages = templates.template_path_to_string("templates/messages.html", &0).await?;
    let content = counter + &messages;
    Ok(Html(
        templates
            .template_path_to_string("templates/index.html", &Index { content })
            .await?,
    ))
}

async fn click(State(mut state): State<AppState>, State(mut templates): State<Templates>) -> Result<impl IntoResponse, HttpError> {
    let clicks = state.click().await;
    info!("click, new counter: {}", clicks);
    Ok(Html(
        templates
            .template_path_to_string("templates/counter.html", &Counter { clicks })
            .await?,
    ))
}

async fn broadcast_message(
    messages_dao: &db::messages::Dao,
    templates: &mut Templates,
    websockets: &mut WebSockets,
    id: u64,
) -> anyhow::Result<()> {
    if let Some(message) = messages_dao.get_by_id(id).await {
        let message = templates
            .template_path_to_string(
                "templates/new-message.html",
                &NewMessage {
                    content: format!("{}: {}", message.sender, message.message),
                },
            )
            .await
            .map_err(|e| anyhow!("error rendering template to respond to websocket message: {:?}", e))?;

        websockets
            .broadcast(message)
            .await
            .map_err(|e| anyhow!("error responding to websocket message: {:?}", e))?;
    }
    Ok(())
}
