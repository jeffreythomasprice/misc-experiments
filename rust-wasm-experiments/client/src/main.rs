#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use log::*;

use leptos::{html::Input, *};
use serde::{de::DeserializeOwned, Serialize};
use shared::models::messages::{
    ClientWebsocketMessage, CreateClientRequest, CreateClientResponse, ServerWebsocketMessage,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;

mod fetch;
use fetch::*;

use crate::websockets::WebSocket;

mod websockets;

const HOST: &str = "127.0.0.1:8001";

#[derive(Debug, Clone)]
struct MessageWithId(u32, ServerWebsocketMessage);

#[component]
fn Message(cx: Scope, message: ServerWebsocketMessage) -> impl IntoView {
    view! {
        cx,
        <div>{format!("{message:?}")}</div>
    }
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    let input_node_ref: NodeRef<Input> = create_node_ref(cx);
    let (input_value, set_input_value) = create_signal(cx, "".to_string());

    let (messages, set_messages) = create_signal(cx, Vec::<MessageWithId>::new());

    let ws = Rc::new(RefCell::<Option<Sender<ClientWebsocketMessage>>>::new(None));

    {
        let ws = ws.clone();
        spawn_local(async move {
            match open_websocket("default initial name").await {
                Ok((sender, mut receiver)) => {
                    ws.replace(Some(sender));

                    spawn_local(async move {
                        let mut next_id = 0;
                        while let Some(msg) = receiver.recv().await {
                            set_messages.update(|messages| {
                                messages.push(MessageWithId(next_id, msg));
                            });
                            next_id += 1;
                        }
                    });
                }
                Err(e) => log::error!("error getting websocket: {e:?}"),
            };
        });
    }

    input_node_ref.on_load(cx, |input| {
        spawn_local(async move {
            input.focus().unwrap();
        });
    });

    let submit = {
        let ws = ws.clone();
        Rc::new(move || {
            let value = input_value.get();
            set_input_value("".to_string());
            input_node_ref().unwrap().focus().unwrap();

            if !value.is_empty() {
                let ws = ws.clone();
                spawn_local(async move {
                    if let Some(ws) = &*ws.borrow() {
                        if let Err(e) = ws.send(ClientWebsocketMessage::Message(value)).await {
                            log::error!("error sending websocket message: {e:?}");
                        }
                    }
                });
            }
        })
    };

    let input_change = move |e| {
        set_input_value(event_target_value(&e));
    };

    let input_key_press = {
        let submit = submit.clone();
        move |e: KeyboardEvent| {
            // enter
            if e.key_code() == 13 {
                submit();
            }
        }
    };

    let submit_button_click = {
        let submit = submit.clone();
        move |_| {
            submit();
        }
    };

    view! {
        cx,
        <div>
            <div>
                <input
                    node_ref=input_node_ref
                    type="text"
                    width="100"
                    on:input=input_change
                    on:keypress=input_key_press
                    prop:value=input_value
                />
                <button on:click=submit_button_click>Submit</button>
            </div>
            <For
                each=messages
                key=|message| { message.0 }
                view=move |cx, msg| view! {cx, <Message message=msg.1/>}
            />
        </div>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    mount_to_body(|cx| {
        view! {cx, <App/>}
    })
}

async fn open_websocket(
    name: &str,
) -> Result<
    (
        Sender<ClientWebsocketMessage>,
        Receiver<ServerWebsocketMessage>,
    ),
    JsValue,
> {
    let client = create_client(&CreateClientRequest {
        name: name.to_string(),
    })
    .await?;
    let (sender, receiver) =
        WebSocket::new_channels(format!("ws://{HOST}/client/ws").as_str()).await?;
    sender
        .send(ClientWebsocketMessage::Authenticate(client.token))
        .await
        .or_else(|e| Err::<_, JsValue>(format!("error sending auth message: {e:?}").into()))?;
    Ok((sender, receiver))
}

async fn create_client(request: &CreateClientRequest) -> Result<CreateClientResponse, JsValue> {
    json_request_response("POST", "/client", request).await
}

async fn json_request_response<RequestType, ResponseType>(
    method: &str,
    uri: &str,
    request: &RequestType,
) -> Result<ResponseType, JsValue>
where
    RequestType: Serialize,
    ResponseType: DeserializeOwned,
{
    let base_url = format!("http://{HOST}");
    let url = if uri.starts_with('/') {
        format!("{base_url}{uri}")
    } else {
        format!("{base_url}/{uri}")
    };

    let response = RequestBuilder::new()
        .method(method)
        .url(url.as_str())
        .json(request)
        .map_err(|e| -> JsValue { format!("{e}").into() })?
        .build()?
        .launch()
        .await?;
    let response_body: ResponseType = response.json().await?;
    Ok(response_body)
}
