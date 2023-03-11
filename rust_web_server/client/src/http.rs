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

#[derive(Debug)]
pub enum Error {
    Js(JsValue),
    SerdeJson(serde_json::Error),
    NeedsLogin,
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

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Js(value.into())
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Error::Js(value.into())
    }
}

pub fn is_logged_in() -> Result<bool, JsValue> {
    Ok(get_auth_header()?.is_some())
}

pub async fn login(username: &str, password: &str) -> Result<(), Error> {
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

pub async fn get_users() -> Result<Vec<UserResponse>, Error> {
    Ok(make_request_with_json_response(&new_authenticated_request(
        "/api/users",
        "GET",
        &assert_auth_header()?,
    )?)
    .await?)
}

// TODO create, update, delete user

async fn make_request_with_json_response<ResponseT>(request: &Request) -> Result<ResponseT, Error>
where
    ResponseT: for<'de> Deserialize<'de>,
{
    request.headers().set("Accepts", "application/json")?;
    let response = JsFuture::from(get_window()?.fetch_with_request(&request))
        .await?
        .dyn_into::<Response>()?;
    trace!("status code {}", response.status_text());
    match response.status() {
        401 => {
            trace!("needs auth");
            Err(Error::NeedsLogin)
        }
        _ => {
            let response_body: ResponseT =
                serde_wasm_bindgen::from_value(JsFuture::from(response.json()?).await?)?;
            Ok(response_body)
        }
    }
}

fn new_authenticated_request(
    url: &str,
    method: &str,
    auth_header: &str,
) -> Result<Request, JsValue> {
    common_authenticated_request(url, method, auth_header, |r| Ok(r), |r| Ok(r))
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
    Ok(common_authenticated_request(
        url,
        method,
        auth_header,
        |r| Ok(r.body(Some(&body))),
        |r| {
            r.headers().set("Content-Type", "application/json")?;
            Ok(r)
        },
    )?)
}

fn common_authenticated_request<InitF, RequestF>(
    url: &str,
    method: &str,
    auth_header: &str,
    init_f: InitF,
    request_f: RequestF,
) -> Result<Request, JsValue>
where
    InitF: Fn(&mut RequestInit) -> Result<&mut RequestInit, JsValue>,
    RequestF: Fn(Request) -> Result<Request, JsValue>,
{
    let result = Request::new_with_str_and_init(url, init_f(RequestInit::new().method(method))?)?;
    result.headers().set("Authorization", auth_header)?;
    Ok(request_f(result)?)
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
