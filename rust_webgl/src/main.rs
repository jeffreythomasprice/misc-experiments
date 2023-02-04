mod app_states;
mod dom_utils;
mod fetch_utils;
mod futures_app_state;

use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::try_join;
use gloo_console::*;
use rand::{rngs::ThreadRng, Rng};
use std::panic;
use web_sys::WebGl2RenderingContext;

use crate::app_states::*;
use crate::fetch_utils::*;
use crate::futures_app_state::*;

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
	if let Err(e) = run_state_machine(|| {
		Ok(Rc::new(RefCell::new(PendingFutureState::new(|| async {
			log!("TODO about to start loading data");
			try_join!(
				fetch_string("assets/shader.vert"),
				fetch_string("assets/shader.frag")
			)?;
			log!("TODO data load complete");
			// TODO JEFF do something with shaders

			let data = Rc::new(RefCell::new(Data::new()?));
			let next_state = Rc::new(RefCell::new(DemoState::new(data)));
			Ok::<AppStateHandle, AppError>(next_state)
		}))))
	}) {
		error!(format!("fatal {e:?}"))
	}
}
