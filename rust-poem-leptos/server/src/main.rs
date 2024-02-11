mod websockets;

use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Cors, Tracing},
    web::{websocket::WebSocket, Data, Json},
    EndpointExt, IntoResponse, Request, Route, Server,
};
use shared::{ClicksResponse, ClientToServerChatMessage, ServerToClientChatMessage};
use std::sync::{Arc, Mutex};
use tracing::*;

#[derive(Clone)]
struct ClicksService {
    count: Arc<Mutex<u64>>,
}

impl ClicksService {
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter("server=trace,poem=debug,debug")
        .init();

    let app = Route::new()
        .at("/click", get(get_clicks).post(click))
        .at("/ws", get(ws))
        .with(Cors::new())
        .with(Tracing)
        .with(AddData::new(ClicksService::new()))
        .with(AddData::new(
            websockets::Service::<ServerToClientChatMessage>::new(),
        ));

    Server::new(TcpListener::bind("127.0.0.1:8001"))
        .run(app)
        .await
}

#[handler]
async fn get_clicks(clicks: Data<&ClicksService>) -> Json<ClicksResponse> {
    let count = clicks.count.lock().unwrap();
    Json(ClicksResponse { clicks: *count })
}

#[handler]
async fn click(clicks: Data<&ClicksService>) -> Json<ClicksResponse> {
    let mut count = clicks.count.lock().unwrap();
    *count += 1;
    Json(ClicksResponse { clicks: *count })
}

#[handler]
async fn ws(
    req: &Request,
    ws: WebSocket,
    ws_service: Data<&websockets::Service<ServerToClientChatMessage>>,
) -> impl IntoResponse {
    debug!("remote_addr={}", req.remote_addr());
    let (result, _sender, mut receiver) = ws_service.on_upgrade::<ClientToServerChatMessage>(ws);

    let ws_service = ws_service.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            match ServerToClientChatMessage::new(&msg) {
                Ok(msg) => ws_service.broadcast(&msg).await,
                Err(e) => error!("failed to make outgoing message to broadcast: {e:?}"),
            };
        }
    });

    result
}
