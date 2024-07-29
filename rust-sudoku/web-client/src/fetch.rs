use crate::dom::window;
use lib::Result;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::Uint8Array, wasm_bindgen, Request, RequestInit, RequestMode, Response};

pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let response = JsFuture::from(window()?.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let response: Response = response.dyn_into().unwrap();
    let response = JsFuture::from(response.array_buffer().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    let response = Uint8Array::new(&response);
    Ok(response.to_vec())
}
