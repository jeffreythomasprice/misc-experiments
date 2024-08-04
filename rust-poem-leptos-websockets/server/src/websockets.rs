use std::fmt::Debug;

use futures_util::{stream, SinkExt, Stream, StreamExt};
use poem::web::websocket::{Message, WebSocketStream};
use serde::{de::DeserializeOwned, Serialize};
use tracing::*;

pub fn split_websocket_stream<OutgoingMessage, IncomingMessage>(
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
