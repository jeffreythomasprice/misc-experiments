use std::{fmt::Debug, sync::Arc};

use futures_util::{stream, SinkExt, Stream, StreamExt};
use poem::{
    handler,
    web::{
        websocket::{Message, WebSocket, WebSocketStream},
        Data, RemoteAddr,
    },
    IntoResponse,
};
use serde::{de::DeserializeOwned, Serialize};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tracing::*;
use uuid::Uuid;

use crate::AppState;

pub struct ActiveWebsocket {
    id: Uuid,
    sender: tokio::sync::mpsc::Sender<WebsocketServerToClientMessage>,
}

#[handler]
pub fn websocket(
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
