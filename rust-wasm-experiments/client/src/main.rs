#![allow(dead_code)]

use log::*;

use leptos::*;
use serde::{de::DeserializeOwned, Serialize};
use shared::models::messages::{ClientHelloRequest, GenericResponse};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

mod fetch;
use fetch::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    let click = move |_| {
        set_count.update(|count| {
            *count += 1;
            ()
        });
    };

    view! {
        cx,
        <div>
            <p>"Clicks: " {move || count()}</p>
            <button on:click=click>"Click me!"</button>
        </div>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();

    spawn_local(async {
        if let Err(e) = example().await {
            console::log_2(&"error making request".into(), &e);
        }
    });

    mount_to_body(|cx| {
        view! {cx, <App/>}
    })
}

async fn example() -> Result<(), JsValue> {
    let response = client_hello("testing".into()).await?;
    info!("TODO JEFF client hello response: {response:?}");

    Ok(())
}

async fn client_hello(name: String) -> Result<GenericResponse, JsValue> {
    Ok(json_request_response("POST", "/client", &ClientHelloRequest { name }).await?)
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
    let base_url = "http://127.0.0.1:8001";
    let url = if uri.starts_with("/") {
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
