use std::{fmt::Debug, pin::Pin};

use anyhow::Result;
use futures::{stream, Sink, SinkExt, Stream, StreamExt};
use leptos::*;
use log::*;
use pharos::{Observable, ObserveConfig};
use serde::{de::DeserializeOwned, Serialize};
use ws_stream_wasm::{WsErr, WsMessage, WsMeta};

pub async fn new_websocket_with_url<OutgoingMessage, IncomingMessage>(
    url: &str,
) -> Result<(
    Pin<Box<dyn Sink<OutgoingMessage, Error = WsErr>>>,
    Pin<Box<dyn Stream<Item = IncomingMessage>>>,
)>
where
    OutgoingMessage: Serialize + 'static,
    IncomingMessage: DeserializeOwned + Debug + 'static,
{
    let (mut ws, wsio) = WsMeta::connect(url, None).await?;

    let mut events = ws.observe(ObserveConfig::default()).await?;

    spawn_local(async move {
        while let Some(e) = events.next().await {
            trace!("websocket event: {e:?}");
        }
    });

    let (sink, stream) = wsio.split();

    let sink = Box::pin(sink.with_flat_map(|msg| {
        stream::iter(match serde_json::to_string(&msg) {
            Ok(msg) => vec![Ok(WsMessage::Text(msg))],
            Err(e) => {
                error!("failed to serialize outgoing websocket message: {e:}");
                Vec::new()
            }
        })
    }));

    let stream = stream
        .filter_map(|msg| async {
            match msg {
                WsMessage::Text(msg) => match serde_json::from_str::<IncomingMessage>(&msg) {
                    Ok(msg) => Some(msg),
                    Err(e) => {
                        error!("failed to deserialize incoming websocket message: {e:?}");
                        None
                    }
                },
                WsMessage::Binary(_) => todo!(),
            }
        })
        .boxed();

    Ok((sink, stream))
}
