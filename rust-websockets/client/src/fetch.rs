use std::{collections::HashMap, ops::Deref};

use serde::de::DeserializeOwned;
use serde::Serialize;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

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

    pub fn post(&mut self) -> &mut Self {
        self.method("POST")
    }

    pub fn put(&mut self) -> &mut Self {
        self.method("PUT")
    }

    pub fn patch(&mut self) -> &mut Self {
        self.method("PATCH")
    }

    pub fn delete(&mut self) -> &mut Self {
        self.method("DELETE")
    }

    pub fn options(&mut self) -> &mut Self {
        self.method("OPTIONS")
    }

    pub fn head(&mut self) -> &mut Self {
        self.method("HEAD")
    }

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

    pub fn body(&mut self, value: &JsValue) -> &mut Self {
        self.init.body(value.into());
        self
    }

    pub fn json<T>(&mut self, value: &T) -> Result<&mut Self, serde_json::Error>
    where
        T: Serialize,
    {
        self.header("Content-Type", "application/json");

        if !self.headers.contains_key("Accept") {
            self.header("Accept", "application/json");
        }

        self.body(&serde_json::to_string(value)?.into());

        Ok(self)
    }

    pub fn build(&self) -> Result<RequestWrapper, JsValue> {
        let url = self
            .url
            .clone()
            .ok_or::<JsValue>("must provide url".into())?;
        let request = Request::new_with_str_and_init(&url, &self.init)?;
        for (key, value) in self.headers.iter() {
            request.headers().set(key, value)?;
        }
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
        &self.0
    }
}

pub struct ResponseWrapper(Response);

impl ResponseWrapper {
    pub async fn text(&self) -> Result<String, JsValue> {
        JsFuture::from(self.0.text()?)
            .await?
            .as_string()
            .ok_or::<JsValue>("output of text() was not a string".into())
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
        &self.0
    }
}