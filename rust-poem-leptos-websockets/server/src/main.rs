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

    let (websocket_sender, _) = tokio::sync::broadcast::channel::<String>(32);

    let app = Route::new()
        .at("/websocket", get(websocket.data(websocket_sender)))
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
    state: Data<&Arc<AppState>>,
    ws: WebSocket,
    remote_addr: &RemoteAddr,
    sender: Data<&tokio::sync::broadcast::Sender<String>>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    debug!("incoming websocket connection from: {remote_addr}, id={id}");

    let sender = sender.clone();
    let mut receiver = sender.subscribe();

    ws.on_upgrade(move |socket| async move {
        let (sink, mut stream) = split_websocket_stream::<
            WebsocketServerToClientMessage,
            WebsocketClientToServerMessage,
        >(socket);

        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                debug!("received incoming websocket message: {msg:?}");
                // TODO send to all open websockets
            }
        });

        // TODO add to list of active websockets

        // tokio::spawn(async move {
        //     while let Ok(msg) = receiver.recv().await {
        //         if let Err(e) = sink.send(Message::Text(format!("{id}:{msg}"))).await {
        //             error!("error sending to websocket client on websocket {id}: {e:?}");
        //             break;
        //         }
        //     }
        // });
    })
}

fn split_websocket_stream<OutgoingMessage, IncomingMessage>(
    socket: WebSocketStream,
) -> (
    std::pin::Pin<Box<dyn Sink<OutgoingMessage, Error = std::io::Error>>>,
    std::pin::Pin<Box<dyn Stream<Item = IncomingMessage> + Send>>,
)
where
    OutgoingMessage: Serialize + Sync + Send + 'static,
    IncomingMessage: DeserializeOwned + Sync + Send + Debug,
{
    let (sink, stream) = socket.split();

    let sink = Box::pin(sink.with_flat_map(|msg| {
        stream::iter(match serde_json::to_string(&msg) {
            Ok(msg) => vec![Ok(poem::web::websocket::Message::Text(msg))],
            Err(e) => {
                error!("failed to serialize outgoing websocket message: {e:}");
                Vec::new()
            }
        })
    }));

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

    (sink, stream)
}
