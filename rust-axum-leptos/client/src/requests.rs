use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response, Window};

#[derive(Debug)]
pub enum Error {
    Js(JsValue),
    Serde(serde_json::Error),
    BadCast(String),
    AssertFailed(String),
    BadStatusCode(u16),
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::Js(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

pub async fn http_request_json_body_json_response<RequestBody, ResponseBody>(
    method: &str,
    path: &str,
    request_body: &RequestBody,
) -> Result<ResponseBody, Error>
where
    RequestBody: Serialize,
    ResponseBody: DeserializeOwned,
{
    // TODO host should be config?
    const HOST: &str = "http://localhost:8001";
    let url = if path.starts_with('/') {
        format!("{HOST}{path}")
    } else {
        format!("{HOST}/{path}")
    };

    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);

    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    headers.append("Accept", "application/json")?;
    opts.headers(&headers);

    let request_body = serde_json::to_string(&request_body)?;
    opts.body(Some(&JsValue::from_str(&request_body)));

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let response = JsFuture::from(window()?.fetch_with_request(&request)).await?;
    let response: Response = response
        .dyn_into()
        .map_err(|_| Error::BadCast("failed to get response object from js runtime".to_string()))?;

    match response.status() {
        200..=299 => {
            let response_body = JsFuture::from(response.text()?).await?;
            let response_body = response_body.as_string().ok_or(Error::BadCast(
                "response body wasn't a string, can't deserialize".to_string(),
            ))?;
            Ok(serde_json::from_str(&response_body)?)
        }
        _ => Err(Error::BadStatusCode(response.status())),
    }
}

fn window() -> Result<Window, Error> {
    web_sys::window().ok_or(Error::AssertFailed("missing window".to_string()))
}
