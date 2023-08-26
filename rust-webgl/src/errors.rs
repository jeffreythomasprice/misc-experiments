use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub struct Error(String);

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        // TODO check if value is a string, or an exception with a stack trace
        Self(format!("JsValue({value:?})"))
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self(format!("{value:?}"))
    }
}
