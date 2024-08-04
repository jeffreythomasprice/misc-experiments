mod websockets;

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use leptos::*;
use log::Level;
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use std::panic;
use websockets::new_websocket_with_url;

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

    mount_to_body(|| view! { <div>"Hello, world!"</div> });

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
