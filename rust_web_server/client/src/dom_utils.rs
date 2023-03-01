use wasm_bindgen::*;
use web_sys::{Event, HtmlInputElement, Storage, Window};

pub fn get_window() -> Result<Window, &'static str> {
    web_sys::window().ok_or("failed to get window")
}

pub fn get_local_storage() -> Result<Storage, JsValue> {
    get_window()?
        .local_storage()?
        .ok_or("failed to get local storage".into())
}

pub fn get_value_from_input_element_event<T>(e: T) -> Option<String>
where
    T: Into<Event>,
{
    e.into()
        .target()?
        .dyn_into::<HtmlInputElement>()
        .ok()
        .and_then(|x| Some(x.value()))
}
