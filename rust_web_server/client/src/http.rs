use base64::Engine;
use log::*;
use serde::{Deserialize, Serialize};
use shared::{
    auth::{self},
    user::UserResponse,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use crate::dom_utils::{get_local_storage, get_window};

const LOCAL_STORAGE_KEY: &str = "jwt";

enum Error {
    Js(JsValue),
    SerdeJson(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Error::Js(value)
    }
}

pub fn is_logged_in() -> Result<bool, JsValue> {
    Ok(get_auth_header()?.is_some())
}

pub async fn login(username: &str, password: &str) -> Result<(), JsValue> {
    let response_body: auth::ResponseBody =
        make_request_with_json_response(&new_authenticated_request(
            "/api/login",
            "POST",
            &get_basic_auth_header(username, password),
        )?)
        .await?;
    trace!("got jwt: {}", response_body.jwt);

    get_local_storage()?.set(LOCAL_STORAGE_KEY, response_body.jwt.as_str())?;
    trace!("saved jwt in local storage");

    Ok(())
}

pub fn logout() -> Result<(), JsValue> {
    Ok(get_local_storage()?.remove_item(LOCAL_STORAGE_KEY)?)
}

pub async fn get_users() -> Result<Vec<UserResponse>, JsValue> {
    Ok(make_request_with_json_response(&new_authenticated_request(
        "/api/users",
        "GET",
        &assert_auth_header()?,
    )?)
    .await?)
}

// TODO create, update, delete user

async fn make_request_with_json_response<ResponseT>(request: &Request) -> Result<ResponseT, JsValue>
where
    ResponseT: for<'de> Deserialize<'de>,
{
    request.headers().set("Accepts", "application/json")?;
    let response = JsFuture::from(get_window()?.fetch_with_request(&request))
        .await?
        .dyn_into::<Response>()?;
    let response_body: ResponseT =
        serde_wasm_bindgen::from_value(JsFuture::from(response.json()?).await?)?;
    Ok(response_body)
}

fn new_authenticated_request(
    url: &str,
    method: &str,
    auth_header: &str,
) -> Result<Request, JsValue> {
    let result = Request::new_with_str_and_init(url, RequestInit::new().method(method))?;
    result.headers().set("Authorization", auth_header)?;
    Ok(result)
}

fn new_authenticated_request_with_json_request_body<RequestT>(
    url: &str,
    method: &str,
    auth_header: &str,
    body: RequestT,
) -> Result<Request, Error>
where
    RequestT: Serialize,
{
    let body: JsValue = serde_json::to_string(&body)?.into();
    let result =
        Request::new_with_str_and_init(url, RequestInit::new().method(method).body(Some(&body)))?;
    result.headers().set("Authorization", auth_header)?;
    result.headers().set("Content-Type", "application/json")?;
    Ok(result)
}

fn assert_auth_header() -> Result<String, JsValue> {
    Ok(get_auth_header()?.ok_or("not authenticated")?)
}

fn get_auth_header() -> Result<Option<String>, JsValue> {
    Ok(get_local_storage()?
        .get(LOCAL_STORAGE_KEY)?
        .and_then(|token| Some(format!("Bearer {}", token))))
}

fn get_basic_auth_header(username: &str, password: &str) -> String {
    // TODO url-encode username and password
    format!(
        "Basic {}",
        base64::engine::general_purpose::URL_SAFE.encode(format!("{username}:{password}"))
    )
}
