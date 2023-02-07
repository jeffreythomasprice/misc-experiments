use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::try_join;
use gloo_console::*;
use memoffset::offset_of;
use rand::{rngs::ThreadRng, Rng};
use std::panic;
use vek::Extent2;
use vek::FrustumPlanes;
use vek::Mat4;
use vek::Rgba;
use vek::Vec2;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::WebGl2RenderingContext;

use lib::*;

mod solid_color_state;
use solid_color_state::*;

// TODO move to lib
fn new_canvas_image<F>(size: Extent2<u32>, f: F) -> Result<HtmlCanvasElement, AppError>
where
	F: Fn(&CanvasRenderingContext2d, Extent2<u32>) -> Result<(), AppError>,
{
	let result = document()?
		.create_element("canvas")?
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.or(Err("failed to cast into the right type"))?;
	result.set_width(size.w);
	result.set_height(size.h);

	let context = result
		.get_context("2d")?
		.ok_or("failed to create canvas 2d context")?
		.dyn_into::<web_sys::CanvasRenderingContext2d>()
		.or(Err("failed to cast into the right type"))?;

	f(&context, size)?;

	Ok(result)
}

struct Data {
	random: ThreadRng,
	shader: Shader,
}

impl Data {
	fn new(shader: Shader) -> Result<Self, AppError> {
		Ok(Self {
			random: rand::thread_rng(),
			shader,
		})
	}
}

struct VertexPos2RGBA<T> {
	pub pos: Vec2<T>,
	pub color: Rgba<T>,
}

struct DemoState {
	data: Rc<RefCell<Data>>,
	ortho_matrix: Mat4<f32>,
	color: Rgba<f32>,
	remaining_time: Duration,
	gl: Option<Rc<WebGl2RenderingContext>>,
	buffer: Option<Buffer>,
	vertex_array: Option<VertexArray>,
}

impl DemoState {
	fn new(data: Rc<RefCell<Data>>) -> Self {
		let color = {
			let data = data.clone();
			let r = &mut data.borrow_mut().random;
			Rgba::new(
				r.gen_range(0f32..=1f32),
				r.gen_range(0f32..=1f32),
				r.gen_range(0f32..=1f32),
				1.0f32,
			)
		};
		let remaining_time = Duration::from_secs_f64(data.borrow_mut().random.gen_range(3.0..=5.0));
		Self {
			data,
			ortho_matrix: Mat4::identity(),
			color,
			remaining_time,
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

		let shader = &self.data.borrow().shader;
		let position_attribute = &shader.get_attributes_by_name()["position_attribute"];
		let color_attribute = &shader.get_attributes_by_name()["color_attribute"];

		let buffer = Buffer::new(gl.clone(), BufferTarget::Array)?;
		buffer.bind();
		unsafe {
			let data = vec![
				VertexPos2RGBA {
					pos: Vec2::new(50f32, 50f32),
					color: Rgba::new(1f32, 0f32, 0f32, 1f32),
				},
				VertexPos2RGBA {
					pos: Vec2::new(300f32, 50f32),
					color: Rgba::new(0f32, 1f32, 0f32, 1f32),
				},
				VertexPos2RGBA {
					pos: Vec2::new(50f32, 300f32),
					color: Rgba::new(0f32, 0f32, 1f32, 1f32),
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
		gl.enable_vertex_attrib_array(position_attribute.location);
		gl.vertex_attrib_pointer_with_i32(
			position_attribute.location,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			std::mem::size_of::<VertexPos2RGBA<f32>>() as i32,
			offset_of!(VertexPos2RGBA<f32>, pos) as i32,
		);
		gl.enable_vertex_attrib_array(color_attribute.location);
		gl.vertex_attrib_pointer_with_i32(
			color_attribute.location,
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

	fn resize(&mut self, size: Extent2<i32>) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();

		gl.viewport(0, 0, size.w, size.h);

		self.ortho_matrix = Mat4::<f32>::orthographic_rh_zo(FrustumPlanes {
			left: 0f32,
			right: size.w as f32,
			bottom: size.h as f32,
			top: 0f32,
			near: -1f32,
			far: 1f32,
		});

		Ok(())
	}

	fn render(&mut self) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();

		gl.clear_color(self.color.r, self.color.g, self.color.b, self.color.a);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

		let shader = &self.data.borrow().shader;
		shader.bind();

		let projection_matrix_uniform = &shader.get_uniforms_by_name()["projection_matrix_uniform"];
		gl.uniform_matrix4fv_with_f32_array(
			Some(&projection_matrix_uniform.location),
			false,
			&self.ortho_matrix.into_col_array(),
		);

		let modelview_matrix_uniform = &shader.get_uniforms_by_name()["modelview_matrix_uniform"];
		gl.uniform_matrix4fv_with_f32_array(
			Some(&modelview_matrix_uniform.location),
			false,
			&Mat4::identity()
				.translated_2d(Vec2::new(50f32, 300f32))
				.into_col_array(),
		);

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
			Rc::new(RefCell::new(SolidColorState::new(Rgba::new(
				0.2f32, 0.2f32, 0.2f32, 1.0f32,
			)))),
			|gl| async move {
				let gl = gl.clone();
				let (vertex_source, fragment_source) = try_join!(
					fetch_string("assets/shader.vert"),
					fetch_string("assets/shader.frag")
				)?;
				let shader = Shader::new(gl, &vertex_source, &fragment_source)?;

				// TODO make a texture out of this
				new_canvas_image(Extent2::new(300, 300), |context, size| {
					context.set_fill_style(&"red".into());
					context.fill_rect(0f64, 0f64, size.w as f64, size.h as f64);
					Ok(())
				})?;

				let data = Rc::new(RefCell::new(Data::new(shader)?));
				let next_state = Rc::new(RefCell::new(DemoState::new(data)));
				Ok::<AppStateHandle, AppError>(next_state)
			},
		))))
	}) {
		error!(format!("fatal {e:?}"))
	}
}
