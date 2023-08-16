use std::{collections::HashMap, ops::Deref};

use console_log;
use log::*;
use serde::de::DeserializeOwned;
use shared::JsonResponse;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{console, Request, RequestInit, RequestMode, Response};
use yew::prelude::*;

mod fetch;
use fetch::*;

#[function_component]
fn App() -> Html {
    html! {
        <div>
            <p>{ "Hello, World!" }</p>
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

    yew::Renderer::<App>::new().render();
}

async fn example() -> Result<(), JsValue> {
    let response = RequestBuilder::new()
        .get()
        .url("http://127.0.0.1:8001/")
        .header("Accept", "text/plain")
        .build()?
        .launch()
        .await?;
    info!(
        "TODO JEFF status: {} {}",
        response.status(),
        response.status_text()
    );
    let response_body = response.text().await?;
    info!("TODO JEFF response body: {response_body}");

    let response = RequestBuilder::new()
        .get()
        .url("http://127.0.0.1:8001/json")
        .header("Accept", "application/json")
        .build()?
        .launch()
        .await?;
    info!(
        "TODO JEFF status: {} {}",
        response.status(),
        response.status_text()
    );
    let response_body: JsonResponse = response.json().await?;
    info!("TODO JEFF response body: {response_body:?}");

    Ok(())
}
