use std::{rc::Rc, time::Duration};

use futures::Future;
use gloo_console::*;
use std::sync::mpsc::{self, Receiver};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;

use crate::app_states::*;

// TODO future app state shouldn't have a default render behavior, should defer to another state

enum Status<T> {
	Pending,
	Complete(T),
	ErrorUnreported(AppError),
	ErrorReported,
}

pub struct PendingFutureState<F, Fut>
where
	F: Fn(Rc<WebGl2RenderingContext>) -> Fut,
	Fut: Future<Output = Result<AppStateHandle, AppError>>,
{
	data_factory: Rc<F>,
	status: Status<AppStateHandle>,
	next_status: Option<Receiver<Status<AppStateHandle>>>,
	gl: Option<Rc<WebGl2RenderingContext>>,
}

impl<F, Fut> PendingFutureState<F, Fut>
where
	F: Fn(Rc<WebGl2RenderingContext>) -> Fut,
	Fut: Future<Output = Result<AppStateHandle, AppError>>,
{
	pub fn new(data_factory: F) -> PendingFutureState<F, Fut> {
		PendingFutureState {
			data_factory: Rc::new(data_factory),
			status: Status::Pending,
			next_status: None,
			gl: None,
		}
	}
}

impl<F, Fut> AppState for PendingFutureState<F, Fut>
where
	F: Fn(Rc<WebGl2RenderingContext>) -> Fut + 'static,
	Fut: Future<Output = Result<AppStateHandle, AppError>>,
{
	fn activate(&mut self, gl: Rc<WebGl2RenderingContext>) -> AppResult<()> {
		self.status = Status::Pending;
		self.gl = Some(gl.clone());

		let data_factory = self.data_factory.clone();
		let (sender, receiver) = mpsc::channel();
		{
			let gl = gl.clone();
			spawn_local(async move {
				match data_factory(gl).await {
					Ok(result) => {
						sender.send(Status::Complete(result)).unwrap();
					}
					Err(e) => {
						sender.send(Status::ErrorUnreported(e)).unwrap();
					}
				}
			});
		}
		self.next_status = Some(receiver);

		Ok(())
	}

	fn deactivate(&mut self) -> AppResult<()> {
		Ok(())
	}

	fn resize(&mut self, width: i32, height: i32) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();
		gl.viewport(0, 0, width, height);
		Ok(())
	}

	fn render(&mut self) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();
		gl.clear_color(0.5f32, 0.5f32, 0.5f32, 1.0f32);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
		Ok(())
	}

	fn update(&mut self, _time: Duration) -> AppResult<Option<AppStateHandle>> {
		// see if we are transitioning to a new state
		if let Some(next_status) = &self.next_status {
			if let Ok(value) = next_status.try_recv() {
				self.status = value;
			}
		}

		// if we have data, go to the next app state
		Ok(match &self.status {
			Status::Complete(result) => Some(result.clone()),
			Status::ErrorUnreported(e) => {
				error!(format!("data init failed: {e:?}"));
				self.status = Status::ErrorReported;
				None
			}
			_ => None,
		})
	}
}