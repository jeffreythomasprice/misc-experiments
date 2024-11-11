mod websockets;

use std::panic;

use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use leptos::*;
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

    // TODO move into the post-name stuff, and use the actual name they enter as the hello
    let websocket_sender = create_local_resource(
        || (),
        |_| async {
            match websockets::connect::<WebsocketClientToServerMessage, WebsocketServerToClientMessage>("http://localhost:8001/ws").await {
                Ok((mut sender, mut receiver)) => {
                    spawn_local(async move {
                        while let Some(message) = receiver.next().await {
                            trace!("received message from websocket: {:?}", message);
                        }
                        // TODO put message on screen
                    });

                    if let Err(e) = sender
                        .send(WebsocketClientToServerMessage::Hello {
                            name: "TODO name here".to_owned(),
                        })
                        .await
                    {
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
    );

    mount_to_body(move || {
        view! {
            <App send_message=move |message| {
                spawn_local(async move {
                    send_message_to_websocket(websocket_sender().flatten(), message).await;
                });
            } />
        }
    })
}

#[component]
fn App(#[prop(into)] send_message: Callback<String, ()>) -> impl IntoView {
    let (name, set_name) = create_signal(None);

    view! {
        {move || match name() {
            Some(name) => {
                view! { <Messages name=name send_message=send_message /> }
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
fn Messages(name: String, #[prop(into)] send_message: Callback<String, ()>) -> impl IntoView {
    let (message, set_message) = create_signal("".to_owned());

    // TODO show a list of received messages

    view! {
        <div>Name: {name}</div>
        <form on:submit=move |e| {
            e.prevent_default();
            let result = message();
            let result = result.trim();
            if !result.is_empty() {
                send_message(result.to_owned());
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
