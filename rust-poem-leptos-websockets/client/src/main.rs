mod api;
mod constants;
mod websockets;

use anyhow::{anyhow, Result};
use api::APIService;
use chrono::{DateTime, Utc};
use constants::{BASE_URL, WS_URL};
use futures::{Sink, SinkExt, StreamExt};
use leptos::*;
use leptos_router::{Route, Router, Routes, A};
use log::Level;
use log::*;
use shared::{LogInRequest, WebsocketClientToServerMessage, WebsocketServerToClientMessage};
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
#[allow(non_snake_case)]
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

#[component]
#[allow(non_snake_case)]
fn LoginForm(api_service: APIService) -> impl IntoView {
    let (username, set_username) = create_signal("".to_owned());
    let (password, set_password) = create_signal("".to_owned());

    let log_in_action = create_action(move |request: &LogInRequest| {
        let request = request.clone();
        {
            let api_service = api_service.clone();
            async move {
                match api_service.log_in(&request).await {
                    Ok(response) => {
                        // TODO pass this up the chain
                        debug!("TODO login response: {response:?}");
                    }
                    Err(e) => {
                        // TODO put error message on screen
                        error!("error logging in: {e:?}");
                    }
                };
            }
        }
    });

    view! {
        <form on:submit=move |e| {
            e.prevent_default();
            log_in_action
                .dispatch(LogInRequest {
                    username: username.get(),
                    password: password.get(),
                });
        }>
            <label for="username">Username</label>
            <input
                name="username"
                type="text"
                placeholder="Username"
                prop:value=username
                on:input=move |e| set_username.set(event_target_value(&e))
            />
            <label for="password">Password</label>
            <input
                name="password"
                type="password"
                placeholder="Password"
                prop:value=password
                on:input=move |e| set_password.set(event_target_value(&e))
            />
            <button type="submit">Log In</button>
        </form>
    }
}

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let api_service = APIService::new(BASE_URL);

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

    // TODO testing
    {
        let api_service = api_service.clone();
        create_resource(
            || (),
            move |_| {
                let api_service = api_service.clone();
                async move {
                    if let Err(e) = api_service.list_users().await {
                        error!("error listing users: {e:?}");
                    }
                }
            },
        );
    }

    // TODO nav menu? or just redirect from not found back to login? redirect from not logged in to login?

    mount_to_body(move || {
        view! {
            <Router>
                <div class="rounded-t-lg overflow-hidden border-t border-l border-r border-gray-400 p-4">
                    <ul class="flex border-b">
                        <li class="mb-px mr-1">
                            <A
                                // TODO how to get selected vs unselected class? aria-current="page" isn't good enough because I can't make the tailwind plugin work
                                class="bg-white inline-block py-2 px-4 font-semibold text-blue-700 border-l border-t border-r rounded-t"
                                href="/messages"
                            >
                                Messages
                            </A>
                        </li>
                        <li class="mr-1">
                            <A
                                class="bg-white inline-block py-2 px-4 font-semibold text-gray-500"
                                href="/login"
                            >
                                Login
                            </A>
                        </li>
                    </ul>
                </div>
                <Routes>
                    <Route
                        path="/messages"
                        view=move || {
                            view! {
                                <Messages
                                    messages=messages
                                    on_submit={
                                        let websocket_sink = websocket_sink.clone();
                                        move |msg| {
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
                                    }
                                />
                            }
                        }
                    />

                    <Route
                        path="/login"
                        view=move || view! { <LoginForm api_service=api_service.clone()/> }
                    />

                    // TODO create user form
                    // TODO page to see when logged in with logout button

                    <Route path="/*any" view=|| view! { <div>Not found</div> }/>
                </Routes>
            </Router>
        }
    });

    Ok(())
}

// TODO rename me? move me?
async fn websocket_demo(
    output: impl Fn(WebsocketServerToClientMessage) + 'static,
) -> Result<
    std::pin::Pin<Box<dyn Sink<WebsocketClientToServerMessage, Error = ws_stream_wasm::WsErr>>>,
> {
    let (sink, mut stream) = new_websocket_with_url::<
        WebsocketClientToServerMessage,
        WebsocketServerToClientMessage,
    >(WS_URL)
    .await?;

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            info!("received message from websocket: {msg:?}");
            output(msg);
        }
    });

    Ok(sink)
}
