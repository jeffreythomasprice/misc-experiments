// TODO rename this file

use wasm_bindgen::*;
use web_sys::{Event, HtmlInputElement};

pub fn get_value_from_input_element<T>(e: T) -> Option<String>
where
    T: Into<Event>,
{
    e.into()
        .target()?
        .dyn_into::<HtmlInputElement>()
        .ok()
        .and_then(|x| Some(x.value()))
}
