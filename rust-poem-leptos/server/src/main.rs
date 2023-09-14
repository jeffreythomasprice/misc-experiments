use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use metadata::LevelFilter;
use poem::{
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{AddData, Cors, Tracing},
    post,
    web::{websocket::WebSocket, Data, Json, RemoteAddr},
    EndpointExt, IntoResponse, Route, Server,
};
use shared::{
    models::{ClientHelloRequest, ClientHelloResponse},
    websockets::{Message, WebSocketChannel},
};
use tokio::spawn;
use tracing::*;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct ClientService {}

impl ClientService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn new_client(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[handler]
fn client_hello(
    client_service: Data<&ClientService>,
    Json(request_body): Json<ClientHelloRequest>,
) -> (StatusCode, Json<ClientHelloResponse>) {
    debug!(request_body.name, "client hello");

    let client_id = client_service.new_client();

    (
        StatusCode::OK,
        Json(ClientHelloResponse {
            client_id: client_id.to_string(),
        }),
    )
}

#[handler]
fn websocket(ws: WebSocket, remote_addr: &RemoteAddr) -> impl IntoResponse {
    shared::websockets::server::handler(ws, remote_addr, move |mut stream| {
        let (sender, mut receiver) = stream.split();

        spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    Ok(Message::Text(value)) => {
                        debug!("TODO JEFF got text message from client, {}", value)
                    }
                    Ok(Message::Binary(value)) => debug!(
                        "TODO JEFF got binary message from client, {} bytes",
                        value.len()
                    ),
                    Err(_e) => error!("TODO JEFF error from websocket"),
                }
            }
        });

        spawn(async move {
            if let Err(e) = sender
                .send(Message::Text(
                    "TODO JEFF test message from server".to_string(),
                ))
                .await
            {
                error!("TODO JEFF error sending test message: {e:?}");
            }
        });
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .parse("")?
                .add_directive("hyper=info".parse()?)
                .add_directive("tungstenite=info".parse()?)
                .add_directive("tokio_tungstenite=info".parse()?),
        )
        .init();

    let client_service = ClientService::new();

    let client_api = Route::new().at("/", post(client_hello));

    let app = Route::new()
        .nest("/client", client_api)
        .at("/ws", get(websocket))
        .with(AddData::new(client_service))
        .with(Tracing)
        .with(Cors::new());

    Server::new(TcpListener::bind("127.0.0.1:8001"))
        .name("hello-world")
        .run(app)
        .await?;

    Ok(())
}
