use lib::Result;
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
        .create_element("canvas")
        .map_err(|e| format!("{e:?}"))?
        .dyn_into()
        .map_err(|_| "created a canvas element, but it wasn't the expected type")?)
}
