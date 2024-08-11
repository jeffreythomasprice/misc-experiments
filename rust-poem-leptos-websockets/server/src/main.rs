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
    listener::TcpListener,
    middleware::{AddData, Cors, Tracing},
    post, EndpointExt, Route, Server,
};
use routes::{
    users::{create_user, list_users, log_in},
    websockets::{websocket, ActiveWebsocket},
};
use uuid::Uuid;

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
        .with(AddData::new(state));
    Server::new(TcpListener::bind(format!(
        "{}:{}",
        std::env::var("ADDRESS")?,
        std::env::var("PORT")?
    )))
    .run(app)
    .await?;

    Ok(())
}

// TODO auth middleware, see https://github.com/poem-web/poem/blob/master/examples/poem/basic-auth/src/main.rs
