mod db;
mod websockets;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use db::connection;
use diesel::prelude::*;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, PgConnection, RunQueryDsl, SelectableHelper,
};
use dotenvy::dotenv;
use futures_util::StreamExt;
use poem::{
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{AddData, Cors, Tracing},
    post,
    web::{websocket::WebSocket, Data, Json, RemoteAddr},
    EndpointExt, IntoResponse, Route, Server,
};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tracing::*;
use uuid::Uuid;
use websockets::split_websocket_stream;

struct ActiveWebsocket {
    id: Uuid,
    sender: tokio::sync::mpsc::Sender<WebsocketServerToClientMessage>,
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

#[handler]
fn websocket(
    Data(state): Data<&Arc<AppState>>,
    ws: WebSocket,
    remote_addr: &RemoteAddr,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    debug!("incoming websocket connection from: {remote_addr}, id={id}");

    let active_websockets = state.active_websockets.clone();
    ws.on_upgrade(move |socket| async move {
        let (sender, mut stream) = split_websocket_stream::<
            WebsocketServerToClientMessage,
            WebsocketClientToServerMessage,
        >(socket);

        {
            let active_websockets = active_websockets.clone();
            tokio::spawn(async move {
                while let Some(msg) = stream.next().await {
                    debug!("received incoming websocket message: {msg:?}");
                    let new_msg =
                        WebsocketServerToClientMessage::Message(format!("response to: {:?}", msg));
                    let active_websockets = active_websockets.lock().unwrap();
                    for (_, ws) in active_websockets.iter() {
                        debug!("sending to {}, msg = {:?}", ws.id, new_msg.clone());
                        let ws = ws.clone();
                        let new_msg = new_msg.clone();
                        tokio::spawn(async move {
                            if let Err(e) = ws.sender.send(new_msg).await {
                                error!("error sending message to websocket: {e:?}");
                            }
                        });
                    }
                }

                debug!("websocket closed {id:?}");
                let mut active_websockets = active_websockets.lock().unwrap();
                active_websockets.remove(&id);
            });
        }

        let mut active_websockets = active_websockets.lock().unwrap();
        active_websockets.insert(id, Arc::new(ActiveWebsocket { id, sender }));
    })
}

#[handler]
fn list_users(
    Data(state): Data<&Arc<AppState>>,
) -> Result<Json<Vec<shared::UserResponse>>, StatusCode> {
    use self::db::schema::users::dsl::*;

    let db = &mut state.db.get().unwrap();
    let results = users
        .select(db::models::User::as_select())
        .load(db)
        .map_err(|e| {
            error!("error selecting users: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(|x| x.into())
        .collect();
    Ok(Json(results))
}

#[handler]
fn create_user(
    Data(state): Data<&Arc<AppState>>,
    Json(request): Json<shared::CreateUserRequest>,
) -> Result<Json<shared::UserResponse>, StatusCode> {
    use self::db::schema::users;

    let request: db::models::UserWithJustUsernameAndPassword = request.into();
    let db = &mut state.db.get().unwrap();
    let result: shared::UserResponse = diesel::insert_into(users::table)
        .values(&request)
        .returning(db::models::User::as_returning())
        .get_result(db)
        .map_err(|e| {
            error!("error inserting new user: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into();
    Ok(Json(result))
}

#[handler]
fn log_in(
    Data(state): Data<&Arc<AppState>>,
    Json(request): Json<shared::LogInRequest>,
) -> Result<Json<shared::UserResponse>, StatusCode> {
    use self::db::schema::users::dsl::*;

    let db = &mut state.db.get().unwrap();
    let results: Vec<db::models::User> = users
        .filter(
            username
                .eq(request.username)
                .and(password.eq(request.password)),
        )
        .limit(1)
        .select(db::models::User::as_select())
        .load(db)
        .map_err(|e| {
            error!("error checking user credentials: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    match &results[..] {
        [user] => {
            debug!("found user with correct credentials");
            Ok(Json((*user).clone().into()))
        }
        _ => {
            debug!("incorrect credentials");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
