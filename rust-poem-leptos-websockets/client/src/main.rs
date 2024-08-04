use std::{fmt::Debug, panic, pin::Pin};

use anyhow::{anyhow, Result};
use futures::{stream, Sink, SinkExt, Stream, StreamExt};
use leptos::*;
use log::Level;
use log::*;
use pharos::{Observable, ObserveConfig};
use serde::{de::DeserializeOwned, Serialize};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
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

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| e.to_string())
        .map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        if let Err(e) = websocket_demo().await {
            error!("error running websocket demo: {e:?}");
        }
    });

    mount_to_body(|| view! { <p>"Hello, world!"</p> });

    Ok(())
}

async fn websocket_demo() -> Result<()> {
    let (mut sink, mut stream) = new_websocket_with_url::<
        WebsocketClientToServerMessage,
        WebsocketServerToClientMessage,
    >("ws://127.0.0.1:8001/websocket")
    .await?;

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            info!("received message from websocket: {msg:?}");
        }
    });

    spawn_local(async move {
        if let Err(e) = sink
            .send(WebsocketClientToServerMessage::Message(
                "Hello, World!".to_owned(),
            ))
            .await
        {
            error!("error sending to websocket: {e:?}");
        }
    });

    Ok(())
}
