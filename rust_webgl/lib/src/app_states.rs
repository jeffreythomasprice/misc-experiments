use std::{cell::RefCell, rc::Rc, time::Duration};

use gloo_console::*;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlContextAttributes, Window};

use crate::dom_utils::*;

#[derive(Debug)]
pub enum AppError {
	Message(String),
	JsValue(JsValue),
}

impl From<JsValue> for AppError {
	fn from(value: JsValue) -> Self {
		AppError::JsValue(value)
	}
}

impl From<&str> for AppError {
	fn from(value: &str) -> Self {
		AppError::Message(value.into())
	}
}

impl From<String> for AppError {
	fn from(value: String) -> Self {
		AppError::Message(value.into())
	}
}

pub type AppResult<T> = std::result::Result<T, AppError>;

pub type AppStateHandle = Rc<RefCell<dyn AppState>>;

pub trait AppState {
	fn activate(&mut self, gl: Rc<WebGl2RenderingContext>) -> AppResult<()>;
	fn deactivate(&mut self) -> AppResult<()>;
	// TODO resize should be Extent2<u32>
	fn resize(&mut self, width: i32, height: i32) -> AppResult<()>;
	fn render(&mut self) -> AppResult<()>;
	fn update(&mut self, time: Duration) -> AppResult<Option<AppStateHandle>>;
}

pub fn run_state_machine<F>(initial_state_factory: F) -> Result<(), AppError>
where
	F: FnOnce() -> Result<AppStateHandle, AppError>,
{
	let window = window()?;
	let document = document()?;
	let body = body()?;

	let canvas = document
		.create_element("canvas")?
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.or(Err("failed to cast into the right type"))?;

	canvas.style().set_property("position", "absolute")?;
	canvas.style().set_property("width", "100%")?;
	canvas.style().set_property("height", "100%")?;
	canvas.style().set_property("left", "0")?;
	canvas.style().set_property("top", "0")?;

	while body.has_child_nodes() {
		body.remove_child(&body.first_child().unwrap())?;
	}
	body.append_child(&canvas)?;

	let context = Rc::new(
		canvas
			.get_context_with_context_options(
				"webgl2",
				&WebGlContextAttributes::new()
					.power_preference(web_sys::WebGlPowerPreference::HighPerformance)
					.antialias(true)
					.depth(false)
					.stencil(false)
					.alpha(true),
			)?
			.ok_or("failed to create canvas context")?
			.dyn_into::<web_sys::WebGl2RenderingContext>()
			.or(Err("failed to cast into the right type"))?,
	);

	// two layers of Rc<RefCell<_>>
	// outer layer is because we have to share a mutable reference to the current state in multiple closures
	// inner layer is because each state might create or share references to other states, and so will return ref counted instances when asked for next state
	let current_state: Rc<RefCell<AppStateHandle>> =
		Rc::new(RefCell::new(initial_state_factory()?));
	{
		let current_state = current_state.clone();
		let current_state = (*current_state).borrow_mut();
		current_state
			.as_ref()
			.borrow_mut()
			.activate(context.clone())?;
	}
	let resize_fn = {
		fn f(
			window: &Window,
			canvas: &HtmlCanvasElement,
			current_state: &Rc<RefCell<AppStateHandle>>,
		) -> Result<(), AppError> {
			let width = window.inner_width()?.as_f64().ok_or("not a number")? as i32;
			let height = window.inner_height()?.as_f64().ok_or("not a number")? as i32;

			log!(format!("resize {width} x {height}"));

			canvas.set_width(width as u32);
			canvas.set_height(height as u32);

			let current_state = (**current_state).borrow_mut();
			current_state.as_ref().borrow_mut().resize(width, height)?;

			Ok(())
		}

		let window = window.clone();
		let canvas = canvas.clone();
		let current_state = current_state.clone();
		move || {
			if let Err(e) = f(&window, &canvas, &current_state) {
				error!(format!("error in resize callback: {e:?}"));
			}
		}
	};
	resize_fn();
	let resize_closure = Closure::<dyn Fn()>::new(resize_fn);
	window.add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())?;
	// intentionally leak so the callback works after the function returns
	resize_closure.forget();

	let animate_closure = Rc::new(RefCell::<Option<Closure<_>>>::new(None));
	let animate_fn = {
		fn f(
			canvas: HtmlCanvasElement,
			context: Rc<WebGl2RenderingContext>,
			current_state: &Rc<RefCell<AppStateHandle>>,
			time: JsValue,
			last_time: &Rc<RefCell<Option<f64>>>,
		) -> Result<(), AppError> {
			let mut current_state = (*current_state).borrow_mut();
			current_state.as_ref().borrow_mut().render()?;

			let now = time.as_f64().ok_or("not a number")?;
			if let Some(last) = *last_time.borrow() {
				let new_state = current_state
					.as_ref()
					.borrow_mut()
					.update(Duration::from_secs_f64((now - last) / 1000f64))?;
				if let Some(new_state) = new_state {
					new_state.as_ref().borrow_mut().activate(context.clone())?;
					current_state.as_ref().borrow_mut().deactivate()?;
					*current_state = new_state.clone();
					new_state
						.as_ref()
						.borrow_mut()
						.resize(canvas.width() as i32, canvas.height() as i32)?;
				}
			}
			last_time.replace(Some(now));

			Ok(())
		}

		let window = window.clone();
		let canvas = canvas.clone();
		let context = context.clone();
		let animate_closure = animate_closure.clone();
		let current_state = current_state.clone();
		let last_time = Rc::new(RefCell::new(None));
		move |time: JsValue| {
			if let Err(e) = f(
				canvas.clone(),
				context.clone(),
				&current_state,
				time,
				&last_time,
			) {
				error!(format!("error doing animate: {e:?}"));
			}

			if let Err(e) = window.request_animation_frame(
				animate_closure
					.borrow()
					.as_ref()
					.unwrap()
					.as_ref()
					.unchecked_ref(),
			) {
				error!(format!("error scheduling next animation: {e:?}"));
			}
		}
	};
	{
		let animate_closure = animate_closure.clone();
		animate_closure.replace(Some(Closure::<dyn Fn(JsValue)>::new(animate_fn)));
	}
	{
		let animate_closure = animate_closure.clone();
		window.request_animation_frame(
			animate_closure
				.borrow()
				.as_ref()
				.unwrap()
				.as_ref()
				.unchecked_ref(),
		)?;
	}

	Ok(())
}
