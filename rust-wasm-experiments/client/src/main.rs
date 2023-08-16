use std::{collections::HashMap, ops::Deref};

use console_log;
use log::*;
use serde::de::DeserializeOwned;
use shared::JsonResponse;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{console, Request, RequestInit, RequestMode, Response};
use yew::prelude::*;

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

pub struct RequestBuilder {
    init: RequestInit,
    url: Option<String>,
    headers: HashMap<String, String>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            init: RequestInit::new().mode(RequestMode::Cors).to_owned(),
            url: None,
            headers: HashMap::new(),
        }
    }

    pub fn method(&mut self, method: &str) -> &mut Self {
        self.init.method(method);
        self
    }

    pub fn get(&mut self) -> &mut Self {
        self.method("GET")
    }

    // TODO other method helpers, POST, PUT, DELETE, PATCH, OPTIONS, HEAD

    pub fn mode(&mut self, mode: RequestMode) -> &mut Self {
        self.init.mode(mode);
        self
    }

    pub fn url(&mut self, url: &str) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    pub fn header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    // TODO body

    pub fn build(&self) -> Result<RequestWrapper, JsValue> {
        let url = self
            .url
            .clone()
            .ok_or::<JsValue>("must provide url".into())?;
        let request = Request::new_with_str_and_init(&url, &self.init)?;
        for (key, value) in self.headers.iter() {
            request.headers().set(key, value)?;
        }
        // TODO body
        Ok(RequestWrapper(request))
    }
}

pub struct RequestWrapper(Request);

impl RequestWrapper {
    pub async fn launch(self) -> Result<ResponseWrapper, JsValue> {
        let window =
            web_sys::window().ok_or::<JsValue>("missing window global, can't fetch".into())?;
        Ok(ResponseWrapper(
            JsFuture::from(window.fetch_with_request(&self.0))
                .await?
                .dyn_into()?,
        ))
    }
}

impl Deref for RequestWrapper {
    type Target = Request;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

pub struct ResponseWrapper(Response);

impl ResponseWrapper {
    pub async fn text(&self) -> Result<String, JsValue> {
        Ok(JsFuture::from(self.0.text()?)
            .await?
            .as_string()
            .ok_or::<JsValue>("output of text() was not a string".into())?)
    }

    pub async fn json<T>(&self) -> Result<T, JsValue>
    where
        T: DeserializeOwned,
    {
        Ok(serde_wasm_bindgen::from_value(
            JsFuture::from(self.0.json()?).await?,
        )?)
    }
}

impl Deref for ResponseWrapper {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}
