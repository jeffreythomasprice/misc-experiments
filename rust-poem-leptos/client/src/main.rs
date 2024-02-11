mod websockets;

use std::sync::{Arc, Mutex};

use leptos::*;
use log::*;
use shared::{ClicksResponse, ClientToServerChatMessage, ServerToClientChatMessage};
use tokio::sync::mpsc::Sender;

use crate::websockets::websocket;

const BASE_URL: &str = "http://127.0.0.1:8001";

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(None);

    let (send_message, set_send_message) = create_signal("".to_string());
    let (messages, set_messages) = create_signal(Vec::<ServerToClientChatMessage>::new());

    create_resource(
        || (),
        move |_| async move {
            async fn f() -> Result<ClicksResponse, reqwest::Error> {
                reqwest::get(format!("{BASE_URL}/click"))
                    .await?
                    .json::<ClicksResponse>()
                    .await
            }
            match f().await {
                Ok(response) => {
                    set_count(Some(response.clicks));
                }
                Err(e) => {
                    error!("error making request: {e:?}");
                }
            }
        },
    );

    let websocket_sender: Arc<Mutex<Option<Sender<ClientToServerChatMessage>>>> =
        Arc::new(Mutex::new(None));
    {
        let websocket_sender = websocket_sender.clone();
        create_resource(
            || (),
            move |_| {
                let websocket_sender = websocket_sender.clone();
                async move {
                    match websocket::<ClientToServerChatMessage, ServerToClientChatMessage>(
                        "ws://127.0.0.1:8001/ws",
                    ) {
                        Ok((sender, mut receiver)) => {
                            spawn_local(async move {
                                while let Some(msg) = receiver.recv().await {
                                    set_messages.update(|messages| {
                                        messages.push(msg);
                                    });
                                }
                            });

                            let mut websocket_sender = websocket_sender.lock().unwrap();
                            websocket_sender.replace(sender);
                        }
                        Err(e) => {
                            error!("websocket error: {e:?}");

                            let mut websocket_sender = websocket_sender.lock().unwrap();
                            websocket_sender.take();
                        }
                    }
                }
            },
        );
    }

    let send_click_request = create_action(move |_: &()| async move {
        async fn f() -> Result<ClicksResponse, reqwest::Error> {
            reqwest::Client::new()
                .post(format!("{BASE_URL}/click"))
                .send()
                .await?
                .json::<ClicksResponse>()
                .await
        }
        match f().await {
            Ok(response) => {
                set_count(Some(response.clicks));
            }
            Err(e) => {
                error!("error making request: {e:?}");
            }
        }
    });

    let send_websocket_message = {
        let websocket_sender = websocket_sender.clone();
        create_action(move |msg: &String| {
            let msg = ClientToServerChatMessage {
                message: msg.clone(),
            };
            let websocket_sender = websocket_sender.clone();
            async move {
                let websocket_sender = websocket_sender.lock().unwrap();
                if let Some(sender) = websocket_sender.as_ref() {
                    if let Err(e) = sender.send(msg).await {
                        error!("error writing to websocket send channel: {e:?}");
                    }
                } else {
                    error!("no websocket sender available, can't send message");
                }
            }
        })
    };

    let on_click = move |_| {
        send_click_request.dispatch(());
    };

    view! {
        {move || match count() {
            Some(count) => view! { <div>Clicks: {count}</div> },
            None => view! { <div>Loading...</div> },
        }}

        <button on:click=on_click>Click Me</button>

        <div>
            <form on:submit=move |e| {
                e.prevent_default();
                send_websocket_message.dispatch(send_message());
                set_send_message("".into());
            }>
                <input
                    type="text"
                    placeholder="Type your message..."
                    autofocus
                    on:input=move |e| {
                        set_send_message(event_target_value(&e));
                    }

                    prop:value=send_message
                />

            </form>
            <For each=move || messages().into_iter().enumerate() key=|(i, _)| *i let:msg>
                <div>
                    {move || {
                        let (_, msg) = msg.clone();
                        msg.message
                    }}

                </div>
            </For>
        </div>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();

    mount_to_body(|| view! { <App/> })
}
