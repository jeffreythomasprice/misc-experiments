use anyhow::{anyhow, Result};
use web_sys::{wasm_bindgen::JsCast, CssStyleDeclaration};

pub fn window() -> Result<web_sys::Window> {
    web_sys::window().ok_or(anyhow!("failed to get window"))
}

pub fn document() -> Result<web_sys::Document> {
    window()?
        .document()
        .ok_or(anyhow!("failed to get document",))
}

pub fn body() -> Result<web_sys::HtmlElement> {
    document()?.body().ok_or(anyhow!("failed to get body"))
}

pub fn create_canvas() -> Result<web_sys::HtmlCanvasElement> {
    let result: web_sys::HtmlCanvasElement = document()?
        .create_element("canvas")
        .map_err(|e| anyhow!("failed to create canvas element: {e:?}"))?
        .dyn_into()
        .map_err(|e| anyhow!("created a canvas element, but it wasn't the expected type: {e:?}"))?;

    let set_style = |style: &CssStyleDeclaration, name: &str, value: &str| -> Result<()> {
        Ok(style
            .set_property(name, value)
            .map_err(|e| anyhow!("failed to set css {} = {} for canvas: {:?}", name, value, e))?)
    };

    set_style(&result.style(), "position", "absolute")?;
    set_style(&result.style(), "width", "100%")?;
    set_style(&result.style(), "height", "100%")?;
    set_style(&result.style(), "left", "0px")?;
    set_style(&result.style(), "top", "0px")?;

    Ok(result)
}
