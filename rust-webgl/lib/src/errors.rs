use std::num::TryFromIntError;

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
        if let Ok::<web_sys::Exception, _>(e) = value.clone().try_into() {
            Self(format!("{}\n{}", e.message(), e.stack()))
        } else {
            Self(format!("JsValue({value:?})"))
        }
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self(format!("{value:?}"))
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self(format!("{value:?}"))
    }
}
