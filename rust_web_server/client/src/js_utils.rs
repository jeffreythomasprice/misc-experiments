use wasm_bindgen::JsValue;

pub fn js_value_to_string(e: JsValue) -> String {
    match e.as_string() {
        Some(s) => s,
        None => format!("{e:?}"),
    }
}
