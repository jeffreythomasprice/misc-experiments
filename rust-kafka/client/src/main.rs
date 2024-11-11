mod websockets;

use std::panic;

use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use leptos::*;
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

    mount_to_body(move || {
        view! { <App /> }
    })
}

#[component]
fn App() -> impl IntoView {
    let (name, set_name) = create_signal(None);

    view! {
        {move || match name() {
            Some(name) => {
                view! { <Messages name=name /> }
            }
            None => view! { <Name done=set_name /> },
        }}
    }
}

#[component]
fn Name(done: WriteSignal<Option<String>>) -> impl IntoView {
    let (name, set_name) = create_signal("".to_owned());

    view! {
        <form on:submit=move |e| {
            e.prevent_default();
            let result = name();
            let result = result.trim();
            if !result.is_empty() {
                done(Some(result.to_owned()));
            }
        }>
            <input
                type="text"
                placeholder="Name"
                autofocus
                prop:value=name
                on:input=move |e| {
                    set_name(event_target_value(&e));
                }
            />
            <button type="submit">OK</button>
        </form>
    }
}

#[component]
fn Messages(name: String) -> impl IntoView {
    let (id, set_id) = create_signal::<Option<String>>(None);
    let (message, set_message) = create_signal("".to_owned());

    let received_message = {
        let set_id = set_id.clone();
        create_action(move |message: &WebsocketServerToClientMessage| {
            let message = message.clone();
            async move {
                trace!("received message from websocket: {:?}", message);
                match message {
                    WebsocketServerToClientMessage::Welcome { id } => set_id(Some(id.to_string())),
                    WebsocketServerToClientMessage::Message {
                        id,
                        timestamp,
                        sender,
                        payload,
                    } => {
                        // TODO put message on screen
                    }
                };
            }
        })
    };

    let websocket_sender = {
        let name = name.clone();
        let received_message = received_message.clone();
        create_local_resource(
            move || name.clone(),
            move |name| async move {
                match websockets::connect::<WebsocketClientToServerMessage, WebsocketServerToClientMessage>("http://localhost:8001/ws")
                    .await
                {
                    Ok((mut sender, mut receiver)) => {
                        spawn_local(async move {
                            while let Some(message) = receiver.next().await {
                                received_message.dispatch(message);
                            }
                        });

                        if let Err(e) = sender.send(WebsocketClientToServerMessage::Hello { name: name.clone() }).await {
                            error!("error sending hello: {:?}", e);
                        }

                        Some(sender)
                    }
                    Err(e) => {
                        error!("error opening websocket, error: {:?}", e);
                        None
                    }
                }
            },
        )
    };

    view! {
        <div>Name: {name}</div>
        <Show when=move || id().is_some()>
            <div>ID: {id}</div>
        </Show>
        <form on:submit=move |e| {
            e.prevent_default();
            let result = message();
            let result = result.trim().to_owned();
            if !result.is_empty() {
                let sender = websocket_sender().flatten();
                spawn_local(async move {
                    send_message_to_websocket(sender, result).await;
                });
                set_message("".to_owned());
            }
        }>
            <input
                type="text"
                placeholder="Message"
                autofocus
                prop:value=message
                on:input=move |e| {
                    set_message(event_target_value(&e));
                }
            />
            <button type="submit">Send</button>
        </form>
    }
}

async fn send_message_to_websocket(sender: Option<Sender<WebsocketClientToServerMessage>>, message: String) {
    if let Some(mut sender) = sender {
        match WebsocketClientToServerMessage::new_message(message.to_string()) {
            Ok(message) => {
                trace!("sending websocket message: {:?}", message);
                if let Err(e) = sender.send(message).await {
                    error!("error sending message to websocket, error: {:?}", e);
                }
            }
            Err(e) => error!("error creating message object, error: {:?}", e),
        }
    } else {
        warn!("websocket sender isn't available");
    }
}
