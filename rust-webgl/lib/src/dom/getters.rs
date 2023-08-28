use web_sys::{window, HtmlElement};

use crate::errors::Result;

pub fn get_window() -> Result<web_sys::Window> {
    Ok(window().ok_or("expected window")?)
}

pub fn get_document() -> Result<web_sys::Document> {
    Ok(get_window()?.document().ok_or("expected document")?)
}

pub fn get_body() -> Result<HtmlElement> {
    Ok(get_document()?.body().ok_or("expected body")?)
}
