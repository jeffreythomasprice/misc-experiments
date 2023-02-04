mod app_states;
mod dom_utils;
mod fetch_utils;

use std::sync::mpsc::{self, Receiver};
use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::try_join;
use gloo_console::*;
use rand::{rngs::ThreadRng, Rng};
use std::panic;
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;

use crate::app_states::*;
use crate::fetch_utils::*;

struct Data {
	random: ThreadRng,
}

impl Data {
	fn new() -> Result<Data, AppError> {
		Ok(Data {
			random: rand::thread_rng(),
		})
	}
}

enum DataLoadStateState {
	Pending,
	Complete,
	Error,
}

struct DataLoadState {
	state: DataLoadStateState,
	next_state: Option<Receiver<DataLoadStateState>>,
}

impl DataLoadState {
	fn new() -> DataLoadState {
		DataLoadState {
			state: DataLoadStateState::Pending,
			next_state: None,
		}
	}
}

impl AppState for DataLoadState {
	fn activate(&mut self, _gl: &WebGl2RenderingContext) -> AppResult<()> {
		self.state = DataLoadStateState::Pending;

		let (sender, receiver) = mpsc::channel();
		spawn_local(async move {
			match try_join!(
				fetch_string("assets/shader.vert"),
				fetch_string("assets/shader.frag")
			) {
				Ok((vert, frag)) => {
					// TODO actually load shaders
					log!(format!("TODO JEFF vertex shader:\n{vert:?}"));
					log!(format!("TODO JEFF fragment shader:\n{frag:?}"));
					sender.send(DataLoadStateState::Complete).unwrap();
					log!("fetch complete");
				}
				Err(e) => {
					sender.send(DataLoadStateState::Error).unwrap();
					error!(format!("fetch error: {e:?}"));
				}
			};
		});
		self.next_state = Some(receiver);

		Ok(())
	}

	fn deactivate(&mut self, _gl: &WebGl2RenderingContext) -> AppResult<()> {
		Ok(())
	}

	fn resize(&mut self, gl: &WebGl2RenderingContext, width: i32, height: i32) -> AppResult<()> {
		gl.viewport(0, 0, width, height);
		Ok(())
	}

	fn render(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()> {
		gl.clear_color(0.5f32, 0.5f32, 0.5f32, 1.0f32);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
		Ok(())
	}

	fn update(
		&mut self,
		_gl: &WebGl2RenderingContext,
		_time: Duration,
	) -> AppResult<Option<AppStateHandle>> {
		// see if we are transitioning to a new state
		if let Some(receiver) = &self.next_state {
			if let Ok(next_state) = receiver.try_recv() {
				self.state = next_state;
			}
		}

		// if we have data, go to the next app state
		Ok(match self.state {
			DataLoadStateState::Complete => {
				log!("data loaded, moving to new state");
				let data = Rc::new(RefCell::new(Data::new()?));
				Some(Rc::new(RefCell::new(DemoState::new(data))))
			}
			_ => None,
		})
	}
}

struct DemoState {
	random: Rc<RefCell<Data>>,
	color: (f32, f32, f32),
	remaining_time: Duration,
}

impl DemoState {
	fn new(data: Rc<RefCell<Data>>) -> DemoState {
		let color = {
			let data = data.clone();
			let r = &mut data.borrow_mut().random;
			(
				r.gen_range(0.0f32..=1.0f32),
				r.gen_range(0.0f32..=1.0f32),
				r.gen_range(0.0f32..=1.0f32),
			)
		};
		let remaining_time = data.borrow_mut().random.gen_range(3.0..=5.0);
		DemoState {
			random: data,
			color,
			remaining_time: Duration::from_secs_f64(remaining_time),
		}
	}
}

impl AppState for DemoState {
	fn activate(&mut self, _gl: &WebGl2RenderingContext) -> AppResult<()> {
		Ok(())
	}

	fn deactivate(&mut self, _gl: &WebGl2RenderingContext) -> AppResult<()> {
		Ok(())
	}

	fn resize(&mut self, gl: &WebGl2RenderingContext, width: i32, height: i32) -> AppResult<()> {
		gl.viewport(0, 0, width, height);
		Ok(())
	}

	fn render(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()> {
		let (red, green, blue) = self.color;
		gl.clear_color(red, green, blue, 1.0f32);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
		Ok(())
	}

	fn update(
		&mut self,
		_gl: &WebGl2RenderingContext,
		time: Duration,
	) -> AppResult<Option<AppStateHandle>> {
		if time > self.remaining_time {
			Ok(Some(Rc::new(RefCell::new(DemoState::new(
				self.random.clone(),
			)))))
		} else {
			self.remaining_time -= time;
			Ok(None)
		}
	}
}

#[async_std::main]
async fn main() {
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	if let Err(e) = run_state_machine(|| Ok(Rc::new(RefCell::new(DataLoadState::new())))) {
		error!(format!("fatal {e:?}"))
	}
}
