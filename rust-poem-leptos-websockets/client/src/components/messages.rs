use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use futures::{Sink, SinkExt, StreamExt};
use leptos::{
    component, create_action, create_resource, create_signal, event_target_value, on_cleanup,
    spawn_local, view, For, IntoView, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate,
};
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use uuid::Uuid;
use ws_stream_wasm::WsErr;

use crate::{api::websockets::new_websocket_with_url, constants::WS_URL};

#[derive(Debug, Clone)]
struct DisplayedMessage {
    id: Uuid,
    received_timestamp: DateTime<Utc>,
    message: String,
}

#[component]
#[allow(non_snake_case)]
pub fn Messages() -> impl IntoView {
    let (messages, set_messages) = create_signal(Vec::<DisplayedMessage>::new());
    let (next_message, set_next_message) = create_signal("".to_owned());

    // TODO doesn't need Arc?
    let (sink, set_sink) = create_signal(Arc::new(Mutex::new(
        None::<Pin<Box<dyn Sink<WebsocketClientToServerMessage, Error = WsErr>>>>,
    )));

    // TODO use a <Suspense> tag

    let next_message_action = create_action(move |request: &String| {
        let request = request.clone();
        async move {
            let sink_value = sink.get_untracked();
            let sink_opt = &mut *sink_value.lock().unwrap();
            if let Some(sink) = sink_opt {
                if let Err(e) = sink
                    .send(WebsocketClientToServerMessage::Message(request.clone()))
                    .await
                {
                    error!("error sending websocket message: {e:?}");
                    // TODO display error to user
                }
            } else {
                warn!("tried to send a message before websocket was ready");
                // TODO display error to user
            }
        }
    });

    on_cleanup(move || {
        let sink_value = sink.get_untracked();
        spawn_local(async move {
            let sink_opt = &mut *sink_value.lock().unwrap();
            if let Some(sink) = sink_opt {
                trace!("closing websocket sink");
                if let Err(e) = sink.close().await {
                    error!("error closing sink: {e:?}");
                    // TODO display error to user
                }
            }
        });
    });

    _ = create_resource(
        || (),
        move |_| async move {
            match new_websocket_with_url::<
                WebsocketClientToServerMessage,
                WebsocketServerToClientMessage,
            >(WS_URL)
            .await
            {
                Ok((sink, mut stream)) => {
                    spawn_local(async move {
                        while let Some(msg) = stream.next().await {
                            info!("received message from websocket: {msg:?}");
                            let WebsocketServerToClientMessage::Message(msg) = msg;
                            // TODO needs to be an action of some kind? accessing value outside of scope
                            set_messages.update(|messages| {
                                messages.push(DisplayedMessage {
                                    id: Uuid::new_v4(),
                                    received_timestamp: Utc::now(),
                                    message: msg,
                                })
                            });
                        }
                        trace!("websocket stream loop done");
                    });

                    set_sink.update(|sink_mutex| {
                        let sink_locked_mutex = &mut *sink_mutex.lock().unwrap();
                        sink_locked_mutex.replace(sink);
                    });
                }
                Err(e) => {
                    error!("error opening websocket: {e:?}");
                    // TODO display error to user
                }
            }
        },
    );

    view! {
        <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
            <form on:submit=move |e| {
                e.prevent_default();
                next_message_action.dispatch(next_message.get());
                set_next_message.set("".to_owned());
            }>
                <input
                    type="text"
                    placeholder="Message"
                    name="message"
                    class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                    prop:value=next_message
                    on:input=move |e| set_next_message.set(event_target_value(&e))
                />
            </form>
            <For
                each=move || { messages.get() }
                key=|msg| { msg.id }
                children=move |msg| {
                    view! {
                        <div>
                            {format!("{}: {}", msg.received_timestamp.to_rfc3339(), msg.message)}
                        </div>
                    }
                }
            />

        </div>
    }
}
