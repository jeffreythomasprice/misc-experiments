use web_sys::{Document, HtmlElement, Window};

pub fn window() -> Result<Window, &'static str> {
	web_sys::window().ok_or("failed to get window")
}

pub fn document() -> Result<Document, &'static str> {
	window()?.document().ok_or("failed to get window")
}

pub fn body() -> Result<HtmlElement, &'static str> {
	document()?.body().ok_or("failed to get body")
}
