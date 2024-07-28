use std::panic;

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use leptos::*;
use log::Level;
use log::*;
use pharos::{Observable, Observe, ObserveConfig};
use ws_stream_wasm::{WsMessage, WsMeta};

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| e.to_string())
        .map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        if let Err(e) = websocket_client().await {
            error!("error launching websocket client: {e:?}");
        }
    });

    mount_to_body(|| view! { <p>"Hello, world!"</p> });

    Ok(())
}

async fn websocket_client() -> Result<()> {
    let (mut ws, wsio) = WsMeta::connect("ws://127.0.0.1:8001/websocket", None).await?;

    let mut events = ws.observe(ObserveConfig::default()).await?;

    spawn_local(async move {
        while let Some(e) = events.next().await {
            debug!("websocket event: {e:?}");
        }
    });

    let (mut sink, mut stream) = wsio.split();

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                WsMessage::Text(msg) => {
                    debug!("received message from websocket: {msg}");
                }
                WsMessage::Binary(msg) => todo!(),
            }
        }
    });

    spawn_local(async move {
        if let Err(e) = sink.send(WsMessage::Text(format!("Hello, World!"))).await {
            error!("error sending to websocket: {e:?}");
        }
    });

    Ok(())
}
