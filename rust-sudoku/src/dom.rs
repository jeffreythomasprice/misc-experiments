use crate::Result;
use web_sys::wasm_bindgen::JsCast;

pub fn window() -> Result<web_sys::Window> {
    Ok(web_sys::window().ok_or("failed to get window")?)
}

pub fn document() -> Result<web_sys::Document> {
    Ok(window()?.document().ok_or("failed to get document")?)
}

pub fn body() -> Result<web_sys::HtmlElement> {
    Ok(document()?.body().ok_or("failed to get body")?)
}

pub fn create_canvas() -> Result<web_sys::HtmlCanvasElement> {
    Ok(document()?
        .create_element("canvas")?
        .dyn_into()
        .map_err(|_| "created a canvas element, but it wasn't the expected type")?)
}

pub fn get_context(
    canvas: &web_sys::HtmlCanvasElement,
) -> Result<web_sys::CanvasRenderingContext2d> {
    Ok(canvas
        .get_context("2d")?
        .ok_or("failed to make 2d context")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "created a context element, but it wasn't the expected type")?)
}
