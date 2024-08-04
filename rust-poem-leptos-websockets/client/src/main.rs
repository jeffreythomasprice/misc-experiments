use std::{fmt::Debug, panic, pin::Pin};

use anyhow::{anyhow, Result};
use futures::{stream, Sink, SinkExt, StreamExt};
use leptos::*;
use log::Level;
use log::*;
use pharos::{Observable, ObserveConfig};
use serde::{de::DeserializeOwned, Serialize};
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use ws_stream_wasm::{WsErr, WsMessage, WsMeta};

struct WebsocketClient<OutgoingMessage> {
    sink: Pin<Box<dyn Sink<OutgoingMessage, Error = WsErr>>>,
}

impl<OutgoingMessage> WebsocketClient<OutgoingMessage>
where
    OutgoingMessage: Serialize + 'static,
{
    pub async fn new_with_url<IncomingMessage>(url: &str) -> Result<Self>
    where
        IncomingMessage: DeserializeOwned + Debug,
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

        let mut stream = stream
            .filter_map(|msg| async {
                match msg {
                    // TODO parse as incoming message
                    WsMessage::Text(msg) => Some(msg),
                    WsMessage::Binary(_) => todo!(),
                }
            })
            .boxed();
        spawn_local(async move {
            while let Some(msg) = stream.next().await {
                match serde_json::from_str::<IncomingMessage>(&msg) {
                    Ok(msg) => info!("received message from websocket: {msg:?}"),
                    Err(e) => error!("failed to deserialize incoming websocket message: {e:?}"),
                };
            }
        });

        Ok(Self { sink })
    }

    pub async fn send(&mut self, msg: OutgoingMessage) {
        if let Err(e) = self.sink.send(msg).await {
            error!("error sending to websocket: {e:?}");
        }
    }
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
    let mut ws: WebsocketClient<WebsocketClientToServerMessage> =
        WebsocketClient::new_with_url::<WebsocketServerToClientMessage>(
            "ws://127.0.0.1:8001/websocket",
        )
        .await?;
    ws.send(WebsocketClientToServerMessage::Message(
        "Hello, World!".to_owned(),
    ))
    .await;
    Ok(())
}
