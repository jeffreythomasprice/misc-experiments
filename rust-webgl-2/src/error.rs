use std::fmt::Display;

use js_sys::wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum Error {
    String(String),
    Js(JsValue),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::String(s) => write!(f, "{}", s),
            Error::Js(js_value) => write!(f, "{:?}", js_value),
        }
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::Js(value)
    }
}
