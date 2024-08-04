mod websockets;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use futures_util::StreamExt;
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    web::{websocket::WebSocket, Data, RemoteAddr},
    EndpointExt, IntoResponse, Route, Server,
};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tracing::*;
use uuid::Uuid;
use websockets::split_websocket_stream;

struct AppState {
    active_websockets:
        Arc<Mutex<HashMap<Uuid, tokio::sync::mpsc::Sender<WebsocketServerToClientMessage>>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "server=trace,poem=debug");
    }
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {
        active_websockets: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = Route::new()
        .at("/websocket", get(websocket))
        .with(Tracing)
        .with(AddData::new(state));
    Server::new(TcpListener::bind("127.0.0.1:8001"))
        .name("hello-world")
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
        let (sink, mut stream) = split_websocket_stream::<
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
                    for (id, sink) in active_websockets.iter() {
                        debug!("sending to {}, msg = {:?}", id, new_msg.clone());
                        let sink = sink.clone();
                        let new_msg = new_msg.clone();
                        tokio::spawn(async move {
                            if let Err(e) = sink.send(new_msg).await {
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
        active_websockets.insert(id, sink);
    })
}
