use std::sync::{mpsc::channel, Arc};

use async_notify::Notify;
use gloo_console::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, HtmlElement, HtmlImageElement, Window};

use crate::AppError;

pub fn window() -> Result<Window, &'static str> {
	web_sys::window().ok_or("failed to get window")
}

pub fn document() -> Result<Document, &'static str> {
	window()?.document().ok_or("failed to get window")
}

pub fn body() -> Result<HtmlElement, &'static str> {
	document()?.body().ok_or("failed to get body")
}

pub async fn new_image_from_url(url: &str) -> Result<HtmlImageElement, AppError> {
	let result = document()?
		.create_element("img")?
		.dyn_into::<HtmlImageElement>()
		.or(Err("failed to cast into the right type"))?;

	let n = Arc::new(Notify::new());
	let (sender, receiver) = channel();

	let onload_fn = {
		let n = n.clone();
		let sender = sender.clone();
		move || {
			if let Err(e) = sender.send(Ok(())) {
				error!(format!("error trying to send success result: {e:?}"));
			}
			n.notify();
		}
	};
	let onload_closure = Closure::<dyn Fn()>::new(onload_fn);
	result.add_event_listener_with_callback("load", onload_closure.as_ref().unchecked_ref())?;

	let onerror_fn = {
		let url = url.to_string();
		let n = n.clone();
		let sender = sender.clone();
		move || {
			if let Err(e) = sender.send(Err(format!("failed to load image from {}", url))) {
				error!(format!("error trying to send error result: {e:?}"));
			}
			n.notify();
		}
	};
	let onerror_closure = Closure::<dyn Fn()>::new(onerror_fn);
	result.add_event_listener_with_callback("error", onerror_closure.as_ref().unchecked_ref())?;

	result.set_src(url);

	n.notified().await;
	receiver.recv().or(Err("recv failure"))??;

	Ok(result)
}
