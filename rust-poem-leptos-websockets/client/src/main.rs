mod websockets;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use futures::{Sink, SinkExt, StreamExt};
use leptos::*;
use log::Level;
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use std::{
    panic,
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use websockets::new_websocket_with_url;

#[derive(Debug, Clone)]
struct DisplayedMessage {
    id: Uuid,
    received_timestamp: DateTime<Utc>,
    message: String,
}

#[component]
fn Messages(
    messages: ReadSignal<Vec<DisplayedMessage>>,
    on_submit: impl Fn(String) + 'static,
) -> impl IntoView {
    let (next_message, set_next_message) = create_signal("".to_owned());

    view! {
        <form on:submit=move |e| {
            e.prevent_default();
            on_submit(next_message.get());
            set_next_message.set("".to_owned());
        }>
            <input
                type="text"
                placeholder="Message"
                name="message"
                prop:value=next_message
                on:input=move |e| set_next_message.set(event_target_value(&e))
            />
        </form>
        <For
            each=move || { messages.get() }
            key=|msg| { msg.id }
            children=move |msg| {
                view! {
                    <div>{format!("{}: {}", msg.received_timestamp.to_rfc3339(), msg.message)}</div>
                }
            }
        />
    }
}

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| e.to_string())
        .map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let (messages, set_messages) = create_signal(Vec::new());

    let websocket_sink = Arc::new(Mutex::new(None));

    {
        let websocket_sink = websocket_sink.clone();
        spawn_local(async move {
            match websocket_demo(move |msg| {
                let WebsocketServerToClientMessage::Message(msg) = msg;
                set_messages.update(|messages| {
                    messages.push(DisplayedMessage {
                        id: Uuid::new_v4(),
                        received_timestamp: Utc::now(),
                        message: msg,
                    })
                });
            })
            .await
            {
                Ok(sink) => {
                    let mut websocket_sink = websocket_sink.lock().unwrap();
                    websocket_sink.replace(sink);
                }
                Err(e) => error!("error running websocket demo: {e:?}"),
            };
        });
    }

    mount_to_body(move || {
        view! {
            <Messages
                messages=messages
                on_submit=move |msg| {
                    info!("submitting outgoing message: {msg}");
                    let websocket_sink = websocket_sink.clone();
                    spawn_local(async move {
                        if let Some(sink) = &mut *websocket_sink.lock().unwrap() {
                            if let Err(e) = sink
                                .send(WebsocketClientToServerMessage::Message(msg))
                                .await
                            {
                                error!("error sending to websocket: {e:?}");
                            }
                        }
                    });
                }
            />
        }
    });

    Ok(())
}

async fn websocket_demo(
    output: impl Fn(WebsocketServerToClientMessage) + 'static,
) -> Result<
    std::pin::Pin<Box<dyn Sink<WebsocketClientToServerMessage, Error = ws_stream_wasm::WsErr>>>,
> {
    let (sink, mut stream) = new_websocket_with_url::<
        WebsocketClientToServerMessage,
        WebsocketServerToClientMessage,
    >("ws://127.0.0.1:8001/websocket")
    .await?;

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            info!("received message from websocket: {msg:?}");
            output(msg);
        }
    });

    Ok(sink)
}
