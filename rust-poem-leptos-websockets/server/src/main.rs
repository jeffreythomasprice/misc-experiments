use std::{
    collections::HashMap,
    fmt::Debug,
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use futures_util::{stream, Sink, SinkExt, Stream, StreamExt};
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    web::{
        websocket::{Message, WebSocket, WebSocketStream},
        Data, RealIp, RemoteAddr,
    },
    EndpointExt, IntoResponse, Route, Server,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tracing::*;
use uuid::Uuid;

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
                        tokio::spawn(async move { if let Err(e) = sink.send(new_msg).await {} });
                    }
                }
            });
        }

        let mut active_websockets = active_websockets.lock().unwrap();
        active_websockets.insert(id, sink);
    })
}

fn split_websocket_stream<OutgoingMessage, IncomingMessage>(
    socket: WebSocketStream,
) -> (
    tokio::sync::mpsc::Sender<OutgoingMessage>,
    std::pin::Pin<Box<dyn Stream<Item = IncomingMessage> + Send>>,
)
where
    OutgoingMessage: Serialize + Sync + Send + 'static,
    IncomingMessage: DeserializeOwned + Sync + Send + Debug,
{
    let (sink, stream) = socket.split();

    let mut sink = Box::pin(sink.with_flat_map(|msg| {
        stream::iter(match serde_json::to_string(&msg) {
            Ok(msg) => vec![Ok(poem::web::websocket::Message::Text(msg))],
            Err(e) => {
                error!("failed to serialize outgoing websocket message: {e:}");
                Vec::new()
            }
        })
    }));

    let (sink_sender, mut sink_receiver) = tokio::sync::mpsc::channel::<OutgoingMessage>(1);
    tokio::spawn(async move {
        while let Some(msg) = sink_receiver.recv().await {
            if let Err(e) = sink.send(msg).await {
                error!("error writing message to websocket: {e:?}");
            }
        }
    });

    let stream = stream
        .filter_map(|msg| async {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(msg) => match serde_json::from_str(&msg) {
                        Ok(msg) => Some(msg),
                        Err(e) => {
                            error!("failed to deserialize incoming websocket message: {e:?}");
                            None
                        }
                    },
                    Message::Binary(_) => todo!(),
                    Message::Ping(_) | Message::Pong(_) | Message::Close(_) => None,
                },
                Err(e) => {
                    error!("error reading incoming message from websocket: {e:?}");
                    None
                }
            }
        })
        .boxed();

    (sink_sender, stream)
}
