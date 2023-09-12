use std::error::Error;

use metadata::LevelFilter;
use poem::{
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{Cors, Tracing},
    post,
    web::{websocket::WebSocket, Json},
    EndpointExt, IntoResponse, Route, Server,
};
use shared::{
    models::{ClientHelloRequest, ClientHelloResponse},
    websockets::{Message, WebSocketChannel},
};
use tokio::{spawn, task::spawn_local};
use tracing::*;
use tracing_subscriber::EnvFilter;

#[handler]
// TODO is tracing macro needed?
// #[tracing::instrument]
fn client_hello(
    Json(request_body): Json<ClientHelloRequest>,
) -> (StatusCode, Json<ClientHelloResponse>) {
    debug!(request_body.name, "client hello");
    (StatusCode::OK, Json(ClientHelloResponse {}))
}

#[handler]
fn websocket(ws: WebSocket) -> impl IntoResponse {
    shared::websockets::server::handler(ws, |mut stream| {
        let _span = span!(Level::DEBUG, "TODO JEFF in impl of websocket handler");

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
                .add_directive("hyper=info".parse()?),
        )
        .init();

    let client_api = Route::new().at("/", post(client_hello));

    let app = Route::new()
        .nest("/client", client_api)
        .at("/ws", get(websocket))
        .with(Tracing)
        .with(Cors::new());

    Server::new(TcpListener::bind("127.0.0.1:8001"))
        .name("hello-world")
        .run(app)
        .await?;

    Ok(())
}
