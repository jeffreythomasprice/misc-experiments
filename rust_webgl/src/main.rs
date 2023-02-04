mod app_states;
mod dom_utils;
mod fetch_utils;
mod futures_app_state;
mod shaders;

use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::try_join;
use gloo_console::*;
use js_sys::Float32Array;
use rand::{rngs::ThreadRng, Rng};
use std::panic;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;
use web_sys::WebGlVertexArrayObject;

use crate::app_states::*;
use crate::fetch_utils::*;
use crate::futures_app_state::*;
use crate::shaders::*;

struct Data {
	random: ThreadRng,
	shader: Shader,
}

impl Data {
	fn new(shader: Shader) -> Result<Data, AppError> {
		Ok(Data {
			random: rand::thread_rng(),
			shader,
		})
	}
}

struct DemoState {
	data: Rc<RefCell<Data>>,
	color: (f32, f32, f32),
	remaining_time: Duration,
	gl: Option<Rc<WebGl2RenderingContext>>,
	buffer: Option<WebGlBuffer>,
	vertex_array: Option<WebGlVertexArrayObject>,
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
			data,
			color,
			remaining_time: Duration::from_secs_f64(remaining_time),
			gl: None,
			buffer: None,
			vertex_array: None,
		}
	}
}

impl AppState for DemoState {
	fn activate(&mut self, gl: Rc<WebGl2RenderingContext>) -> AppResult<()> {
		let gl = gl.clone();
		self.gl = Some(gl.clone());

		let position_attrib_location = self
			.data
			.borrow()
			.shader
			.get_attrib_location("positionAttribute")
			.ok_or("can't find attribute")?;

		let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
		unsafe {
			let data = Float32Array::view(&[-0.5f32, -0.5f32, 0.5f32, -0.5f32, 0.0f32, 0.5f32]);
			gl.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ARRAY_BUFFER,
				&data,
				WebGl2RenderingContext::STATIC_DRAW,
			);
		}
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
		self.buffer = Some(buffer.clone());

		let vertex_array = gl
			.create_vertex_array()
			.ok_or("failed to create vertex array")?;
		gl.bind_vertex_array(Some(&vertex_array));
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
		gl.enable_vertex_attrib_array(position_attrib_location);
		gl.vertex_attrib_pointer_with_i32(
			position_attrib_location,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			0,
			0,
		);
		gl.bind_vertex_array(None);
		self.vertex_array = Some(vertex_array);

		Ok(())
	}

	fn deactivate(&mut self) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();

		if let Some(buffer) = &self.buffer {
			gl.delete_buffer(Some(buffer));
		}
		if let Some(vertex_array) = &self.vertex_array {
			gl.delete_vertex_array(Some(vertex_array));
		}

		Ok(())
	}

	fn resize(&mut self, width: i32, height: i32) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();
		gl.viewport(0, 0, width, height);
		Ok(())
	}

	fn render(&mut self) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();

		let (red, green, blue) = self.color;
		gl.clear_color(red, green, blue, 1.0f32);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

		let shader = &self.data.borrow().shader;
		shader.bind();
		if let Some(vertex_array) = &self.vertex_array {
			gl.bind_vertex_array(Some(vertex_array));
			gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
			gl.bind_vertex_array(None);
		}
		shader.bind_none();

		Ok(())
	}

	fn update(&mut self, time: Duration) -> AppResult<Option<AppStateHandle>> {
		if time > self.remaining_time {
			Ok(Some(Rc::new(RefCell::new(DemoState::new(
				self.data.clone(),
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
		Ok(Rc::new(RefCell::new(PendingFutureState::new(
			|gl| async move {
				let gl = gl.clone();
				let (vertex_source, fragment_source) = try_join!(
					fetch_string("assets/shader.vert"),
					fetch_string("assets/shader.frag")
				)?;
				let shader = Shader::new(gl, &vertex_source, &fragment_source)?;

				let data = Rc::new(RefCell::new(Data::new(shader)?));
				let next_state = Rc::new(RefCell::new(DemoState::new(data)));
				Ok::<AppStateHandle, AppError>(next_state)
			},
		))))
	}) {
		error!(format!("fatal {e:?}"))
	}
}
