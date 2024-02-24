use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum JsInteropError {
    JsError(JsValue),
    CastError(String),
    NotFound(String),
    SerdeWasm(serde_wasm_bindgen::Error),
    WebGl(String),
}

impl From<JsValue> for JsInteropError {
    fn from(value: JsValue) -> Self {
        Self::JsError(value)
    }
}

impl From<serde_wasm_bindgen::Error> for JsInteropError {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::SerdeWasm(value)
    }
}
