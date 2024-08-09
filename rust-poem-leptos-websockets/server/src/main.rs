mod db;
mod routes;
mod service;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use db::connection;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenvy::dotenv;
use poem::{
    get,
    http::StatusCode,
    listener::TcpListener,
    middleware::{AddData, Cors, Tracing},
    post,
    web::Json,
    EndpointExt, IntoResponse, Route, Server,
};
use routes::{
    users::{create_user, list_users, log_in},
    websockets::{websocket, ActiveWebsocket},
};
use shared::ErrorResponse;
use uuid::Uuid;

// TODO move me to a routes module
#[derive(Debug, Clone)]
struct StandardErrorResponse {
    pub status_code: StatusCode,
    pub body: ErrorResponse,
}

impl From<StatusCode> for StandardErrorResponse {
    fn from(value: StatusCode) -> Self {
        Self {
            status_code: value,
            body: ErrorResponse {
                message: match value.canonical_reason() {
                    Some(x) => x.to_owned(),
                    None => value.to_string(),
                },
            },
        }
    }
}

impl IntoResponse for StandardErrorResponse {
    fn into_response(self) -> poem::Response {
        (self.status_code, Json(self.body)).into_response()
    }
}

impl Into<poem::error::Error> for StandardErrorResponse {
    fn into(self) -> poem::error::Error {
        poem::error::Error::from_response(self.into_response())
    }
}

struct AppState {
    db: Pool<ConnectionManager<PgConnection>>,
    active_websockets: Arc<Mutex<HashMap<Uuid, Arc<ActiveWebsocket>>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        db: connection()?,
        active_websockets: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = Route::new()
        .at("/websocket", get(websocket))
        .nest(
            "/users",
            Route::new().at("/", get(list_users).post(create_user)),
        )
        .at("/login", post(log_in))
        .with(Tracing)
        .with(AddData::new(state))
        .with(Cors::new());
    Server::new(TcpListener::bind(format!(
        "{}:{}",
        std::env::var("ADDRESS")?,
        std::env::var("PORT")?
    )))
    .run(app)
    .await?;

    Ok(())
}
