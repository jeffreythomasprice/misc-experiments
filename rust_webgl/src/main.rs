mod app_states;
mod buffer;
mod dom_utils;
mod fetch_utils;
mod futures_app_state;
mod jsarray_utils;
mod shader;
mod vertex_array;

use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::try_join;
use gloo_console::*;
use memoffset::offset_of;
use rand::{rngs::ThreadRng, Rng};
use std::panic;
use web_sys::WebGl2RenderingContext;

use crate::app_states::*;
use crate::buffer::*;
use crate::fetch_utils::*;
use crate::futures_app_state::*;
use crate::jsarray_utils::*;
use crate::shader::*;
use crate::vertex_array::*;

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

struct Vector2<T> {
	pub x: T,
	pub y: T,
}

impl<T> Vector2<T> {
	fn new(x: T, y: T) -> Vector2<T> {
		Vector2 { x, y }
	}
}

struct RGBA<T> {
	pub red: T,
	pub green: T,
	pub blue: T,
	pub alpha: T,
}

impl<T> RGBA<T> {
	fn new(red: T, green: T, blue: T, alpha: T) -> RGBA<T> {
		RGBA {
			red,
			green,
			blue,
			alpha,
		}
	}
}

struct VertexPos2RGBA<T> {
	pub pos: Vector2<T>,
	pub color: RGBA<T>,
}

struct DemoState {
	data: Rc<RefCell<Data>>,
	color: (f32, f32, f32),
	remaining_time: Duration,
	gl: Option<Rc<WebGl2RenderingContext>>,
	buffer: Option<Buffer>,
	vertex_array: Option<VertexArray>,
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

		// TODO helper for getting attributes by name?
		let position_attrib_location = self
			.data
			.borrow()
			.shader
			.get_attrib_location("positionAttribute")
			.ok_or("can't find attribute")?;
		let color_attrib_location = self
			.data
			.borrow()
			.shader
			.get_attrib_location("colorAttribute")
			.ok_or("can't find attribute")?;

		let buffer = Buffer::new(gl.clone(), BufferTarget::Array)?;
		buffer.bind();
		unsafe {
			let data = vec![
				VertexPos2RGBA {
					pos: Vector2::new(-0.5f32, 0.5f32),
					color: RGBA::new(1.0f32, 0.0f32, 0.0f32, 1.0f32),
				},
				VertexPos2RGBA {
					pos: Vector2::new(0.5f32, 0.5f32),
					color: RGBA::new(0.0f32, 1.0f32, 0.0f32, 1.0f32),
				},
				VertexPos2RGBA {
					pos: Vector2::new(0.0f32, -0.5f32),
					color: RGBA::new(0.0f32, 0.0f32, 1.0f32, 1.0f32),
				},
			];
			gl.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ARRAY_BUFFER,
				&data.as_uint8array(),
				WebGl2RenderingContext::STATIC_DRAW,
			);
		}
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
		buffer.bind_none();

		let vertex_array = VertexArray::new(gl.clone())?;
		vertex_array.bind();
		buffer.bind();
		gl.enable_vertex_attrib_array(position_attrib_location);
		gl.vertex_attrib_pointer_with_i32(
			position_attrib_location,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			std::mem::size_of::<VertexPos2RGBA<f32>>() as i32,
			offset_of!(VertexPos2RGBA<f32>, pos) as i32,
		);
		gl.enable_vertex_attrib_array(color_attrib_location);
		gl.vertex_attrib_pointer_with_i32(
			color_attrib_location,
			4,
			WebGl2RenderingContext::FLOAT,
			false,
			std::mem::size_of::<VertexPos2RGBA<f32>>() as i32,
			offset_of!(VertexPos2RGBA<f32>, color) as i32,
		);
		buffer.bind_none();
		vertex_array.bind_none();

		self.buffer = Some(buffer);
		self.vertex_array = Some(vertex_array);

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

		let (red, green, blue) = self.color;
		gl.clear_color(red, green, blue, 1.0f32);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

		let shader = &self.data.borrow().shader;
		shader.bind();
		if let Some(vertex_array) = &self.vertex_array {
			vertex_array.bind();
			gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
			vertex_array.bind_none();
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
