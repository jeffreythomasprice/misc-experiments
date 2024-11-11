mod websockets;

use std::panic;

use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use leptos::*;
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

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

    let send_message = create_action(move |message: &String| send_message_to_websocket(websocket_sender().flatten(), message.to_string()));

    mount_to_body(move || {
        view! {
            <Counter callback=move |message| {
                send_message.dispatch(message)
            } />
        }
    })
}

#[component]
fn Counter(callback: impl Fn(String) + 'static) -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div>Clicks: {count}</div>
        <button on:click=move |_| {
            let new_count = count() + 1;
            set_count(new_count);
            info!("count is now {}", new_count);
            callback(format!("count is now {}", new_count));
        }>Click Me</button>
    }
}

async fn send_message_to_websocket(sender: Option<Sender<WebsocketClientToServerMessage>>, message: String) {
    if let Some(mut sender) = sender {
        let message = WebsocketClientToServerMessage::new_message(message.to_string());
        trace!("sending websocket message: {:?}", message);
        if let Err(e) = sender.send(message).await {
            error!("error sending message to websocket, error: {:?}", e);
        }
    } else {
        warn!("websocket sender isn't available");
    }
}
