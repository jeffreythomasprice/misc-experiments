#![allow(dead_code)]

use log::*;

use leptos::*;
use serde::{de::DeserializeOwned, Serialize};
use shared::models::messages::{ClientWebsocketMessage, CreateClientRequest, CreateClientResponse};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, ErrorEvent, MessageEvent, WebSocket};

mod fetch;
use fetch::*;

const HOST: &str = "127.0.0.01:8001";

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    let click = move |_| {
        set_count.update(|count| {
            *count += 1;
        });
    };

    view! {
        cx,
        <div>
            <p>"Clicks: " {count}</p>
            <button on:click=click>"Click me!"</button>
        </div>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();

    spawn_local(async {
        if let Err(e) = startup().await {
            console::log_2(&"error making request".into(), &e);
        }
    });

    mount_to_body(|cx| {
        view! {cx, <App/>}
    })
}

async fn startup() -> Result<(), JsValue> {
    let response = create_client(&CreateClientRequest {
        name: "my name".to_string(),
    })
    .await?;
    info!("TODO JEFF create client response: {response:?}");

    start_websocket(response.token.clone())?;

    Ok(())
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

fn start_websocket(auth_token: String) -> Result<(), JsValue> {
    let ws = WebSocket::new(format!("ws://{HOST}/client/ws").as_str())?;

    let onopen = {
        let ws = ws.clone();
        Closure::<dyn FnMut()>::new(move || {
            info!("TODO JEFF onopen, implement me");

            // TODO testing send
            match serde_json::to_string(&ClientWebsocketMessage::Authenticate {
                token: auth_token.clone(),
            }) {
                Ok(json) => {
                    if let Err(e) = ws.send_with_str(&json) {
                        log::error!("error sending message: {e:?}");
                    }
                }
                Err(e) => log::error!("error converting message to string: {e:?}"),
            };
            // ws.send_with_str("TODO JEFF testing send").unwrap();
        })
    };
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    // forget, to avoid cleaning at the end of the function to js can call this layer
    onopen.forget();

    let onclose = Closure::<dyn FnMut()>::new(move || {
        info!("TODO JEFF onclose, implement me");
    });
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    // forget, to avoid cleaning at the end of the function to js can call this layer
    onclose.forget();

    let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        info!("TODO JEFF onmessage, implement me, e = {e:?}");
    });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    // forget, to avoid cleaning at the end of the function to js can call this layer
    onmessage.forget();

    let onerror = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        console::log_2(&"TODO JEFF onerror".into(), &e);
        log::error!("TODO JEFF onerror, implement me, e = {:?}", e.to_string());
    });
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    // forget, to avoid cleaning at the end of the function to js can call this layer
    onerror.forget();

    Ok(())
}
