#![allow(dead_code)]

use std::{cell::RefCell, sync::Arc};

use log::*;

use leptos::{html::Input, *};
use serde::{de::DeserializeOwned, Serialize};
use shared::models::messages::{
    ClientWebsocketMessage, CreateClientRequest, CreateClientResponse, ServerWebsocketMessage,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;

mod fetch;
use fetch::*;

use crate::websockets::WebSocket;

mod websockets;

const HOST: &str = "127.0.0.01:8001";

#[component]
fn App(cx: Scope) -> impl IntoView {
    let input_node_ref: NodeRef<Input> = create_node_ref(cx);
    let (input_value, set_input_value) = create_signal(cx, "".to_string());

    let ws = Arc::new(RefCell::<
        Option<WebSocket<ClientWebsocketMessage, ServerWebsocketMessage>>,
    >::new(None));

    {
        let ws = ws.clone();
        spawn_local(async move {
            match open_websocket("default initial name").await {
                Ok(result) => {
                    ws.replace(Some(result));
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
        Arc::new(move || {
            let value = input_value.get();
            set_input_value("".to_string());
            input_node_ref().unwrap().focus().unwrap();

            if !value.is_empty() {
                if let Some(ws) = &*ws.borrow() {
                    if let Err(e) = ws.send(ClientWebsocketMessage::Message(value)) {
                        log::error!("error sending websocket message: {e:?}");
                    }
                }
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
        </div>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    mount_to_body(|cx| {
        view! {cx, <App/>}
    })
}

struct WebSocketEventHandler {}

impl websockets::EventHandler<ServerWebsocketMessage> for WebSocketEventHandler {
    fn onopen(&self) {
        info!("TODO JEFF onopen");
    }

    fn onclose(&self) {
        info!("TODO JEFF onclose");
    }

    fn onerror(&self) {
        log::error!("TODO JEFF onerror");
    }

    fn onmessage(&self, message: ServerWebsocketMessage) {
        info!("TODO JEFF onmessage: {message:?}");
    }
}

async fn open_websocket(
    name: &str,
) -> Result<WebSocket<ClientWebsocketMessage, ServerWebsocketMessage>, JsValue> {
    let client = create_client(&CreateClientRequest {
        name: name.to_string(),
    })
    .await?;
    let ws = WebSocket::new(
        format!("ws://{HOST}/client/ws").as_str(),
        WebSocketEventHandler {},
    )
    .await?;
    match ws.send(ClientWebsocketMessage::Authenticate(client.token)) {
        Ok(_) => Ok(ws),
        Err(e) => {
            ws.close();
            Err(format!("error sending auth message: {e:?}").into())
        }
    }
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
