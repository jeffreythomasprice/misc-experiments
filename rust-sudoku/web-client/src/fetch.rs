use std::sync::Mutex;

use crate::dom::window;
use lib::{graphics::Renderer, Result};
use rusttype::Font;
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

pub async fn fetch_utf8(url: &str) -> Result<String> {
    let bytes = fetch_bytes(url).await?;
    let result = core::str::from_utf8(&bytes)?;
    Ok(result.to_owned())
}

pub async fn load_font_url(url: &str) -> Result<Font<'static>> {
    let bytes = fetch_bytes(url).await?;
    Ok(Font::try_from_vec(bytes).ok_or("failed to parse font".to_string())?)
}

pub async fn load_svg_url<R>(renderer: &Mutex<R>, url: &str) -> Result<R::SVG>
where
    R: Renderer,
{
    let s = fetch_utf8(url).await?;
    let renderer = renderer.lock().unwrap();
    renderer.new_svg(&s)
}
