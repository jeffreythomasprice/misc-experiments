mod constants;
mod websockets;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use constants::{BASE_URL, WS_URL};
use futures::{Sink, SinkExt, StreamExt};
use leptos::*;
use log::Level;
use log::*;
use shared::{
    LogInRequest, UserResponse, WebsocketClientToServerMessage, WebsocketServerToClientMessage,
};
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
fn LoginForm() -> impl IntoView {
    let (username, set_username) = create_signal("".to_owned());
    let (password, set_password) = create_signal("".to_owned());

    let log_in_action = create_action(|request: &LogInRequest| {
        let request = request.clone();
        async move {
            match log_in(&request).await {
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
    create_resource(
        || (),
        |_| async move {
            if let Err(e) = list_users().await {
                error!("error listing users: {e:?}");
            }
        },
    );

    // TODO login form
    // TODO create user form
    // TODO page to see when logged in with logout button

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

            <LoginForm/>
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

// TODO move me?
async fn list_users() -> Result<Vec<UserResponse>> {
    let response = reqwest::get(format!("{}/users", BASE_URL)).await?;
    debug!("list users response: {:?}", response);
    let response_body = response.bytes().await?;
    let response_body = serde_json::from_slice(&response_body)?;
    debug!("list users response body: {:?}", response_body);
    Ok(response_body)
}

// TODO move me?
async fn log_in(request: &LogInRequest) -> Result<UserResponse> {
    // TODO deduplicate this stuff with other requests
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/login", BASE_URL))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(request)?)
        .send()
        .await?;
    debug!("login response: {:?}", response);
    let response_body = response.bytes().await?;
    let response_body = serde_json::from_slice(&response_body)?;
    debug!("login response body: {:?}", response_body);
    Ok(response_body)
}
