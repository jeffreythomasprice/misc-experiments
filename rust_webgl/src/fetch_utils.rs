use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use crate::dom_utils::*;

pub async fn fetch_string(url: &str) -> Result<JsValue, JsValue> {
	let request = Request::new_with_str_and_init(
		url,
		RequestInit::new()
			.method("GET")
			.mode(web_sys::RequestMode::Cors),
	)?;
	let response = JsFuture::from(window()?.fetch_with_request(&request))
		.await?
		.dyn_into::<Response>()?;
	let response_body = JsFuture::from(response.text()?).await?;
	Ok(response_body)
}
