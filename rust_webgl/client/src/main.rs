use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::try_join;
use gloo_console::*;
use memoffset::offset_of;
use rand::{rngs::ThreadRng, Rng};
use std::panic;
use vek::Aabr;
use vek::Extent2;
use vek::FrustumPlanes;
use vek::Mat4;
use vek::Rect;
use vek::Rgba;
use vek::Vec2;
use web_sys::WebGl2RenderingContext;

use lib::*;

mod solid_color_state;
use solid_color_state::*;

struct Data {
	random: ThreadRng,
	shader: Shader,
	texture: Texture2d,
}

impl Data {
	fn new(shader: Shader, texture: Texture2d) -> Result<Self, AppError> {
		Ok(Self {
			random: rand::thread_rng(),
			shader,
			texture,
		})
	}
}

#[derive(Debug, Clone, Copy)]
struct VertexPos2Coord2RGBA<T> {
	pub pos: Vec2<T>,
	pub coord: Vec2<T>,
	pub color: Rgba<T>,
}

struct DemoState {
	data: Rc<RefCell<Data>>,
	ortho_matrix: Mat4<f32>,
	color: Rgba<f32>,
	remaining_time: Duration,
	gl: Option<Rc<WebGl2RenderingContext>>,
	array_buffer: Option<Buffer>,
	element_array_buffer: Option<Buffer>,
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
			array_buffer: None,
			element_array_buffer: None,
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
		let texture_coordinate_attribute =
			&shader.get_attributes_by_name()["texture_coordinate_attribute"];
		let color_attribute = &shader.get_attributes_by_name()["color_attribute"];

		let array_buffer = Buffer::new(gl.clone(), BufferTarget::Array)?;
		array_buffer.bind();
		let element_array_buffer = Buffer::new(gl.clone(), BufferTarget::ElementArray)?;
		element_array_buffer.bind();
		unsafe {
			let size = self.data.borrow().texture.size();
			let mut vertices = Vec::new();
			let mut indices = Vec::new();
			add_rect(
				&mut vertices,
				&mut indices,
				Rect::new(0f32, 0f32, size.w as f32, size.h as f32).into_aabr(),
				Rect::new(0f32, 0f32, 1f32, 1f32).into_aabr(),
				Rgba::new(1f32, 1f32, 1f32, 1f32),
			);
			gl.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ARRAY_BUFFER,
				&vertices.as_uint8array(),
				WebGl2RenderingContext::STATIC_DRAW,
			);
			gl.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
				&indices.as_uint8array(),
				WebGl2RenderingContext::STATIC_DRAW,
			)
		}
		array_buffer.bind_none();
		element_array_buffer.bind_none();

		let vertex_array = VertexArray::new(gl.clone())?;
		vertex_array.bind();
		array_buffer.bind();
		gl.enable_vertex_attrib_array(position_attribute.location);
		gl.vertex_attrib_pointer_with_i32(
			position_attribute.location,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			std::mem::size_of::<VertexPos2Coord2RGBA<f32>>() as i32,
			offset_of!(VertexPos2Coord2RGBA<f32>, pos) as i32,
		);
		gl.enable_vertex_attrib_array(texture_coordinate_attribute.location);
		gl.vertex_attrib_pointer_with_i32(
			texture_coordinate_attribute.location,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			std::mem::size_of::<VertexPos2Coord2RGBA<f32>>() as i32,
			offset_of!(VertexPos2Coord2RGBA<f32>, coord) as i32,
		);
		gl.enable_vertex_attrib_array(color_attribute.location);
		gl.vertex_attrib_pointer_with_i32(
			color_attribute.location,
			4,
			WebGl2RenderingContext::FLOAT,
			false,
			std::mem::size_of::<VertexPos2Coord2RGBA<f32>>() as i32,
			offset_of!(VertexPos2Coord2RGBA<f32>, color) as i32,
		);
		array_buffer.bind_none();
		vertex_array.bind_none();

		self.array_buffer = Some(array_buffer);
		self.element_array_buffer = Some(element_array_buffer);
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

		let sampler_uniform = &shader.get_uniforms_by_name()["sampler_uniform"];
		gl.active_texture(WebGl2RenderingContext::TEXTURE1);
		gl.uniform1i(Some(&sampler_uniform.location), 1);

		if let (texture, Some(vertex_array), Some(element_array_buffer)) = (
			&self.data.borrow().texture,
			&self.vertex_array,
			&self.element_array_buffer,
		) {
			texture.bind();

			vertex_array.bind();
			element_array_buffer.bind();
			gl.draw_elements_with_i32(
				WebGl2RenderingContext::TRIANGLES,
				6,
				WebGl2RenderingContext::UNSIGNED_SHORT,
				0,
			);
			element_array_buffer.bind_none();
			vertex_array.bind_none();

			texture.bind_none();
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
				let (vertex_source, fragment_source) = try_join!(
					fetch_string("assets/shader.vert"),
					fetch_string("assets/shader.frag")
				)?;
				let shader = Shader::new(gl.clone(), &vertex_source, &fragment_source)?;

				let texture =
					Texture2d::new_with_image_url(gl.clone(), "assets/bricks.png").await?;

				let data = Rc::new(RefCell::new(Data::new(shader, texture)?));
				let next_state = Rc::new(RefCell::new(DemoState::new(data)));
				Ok::<AppStateHandle, AppError>(next_state)
			},
		))))
	}) {
		error!(format!("fatal {e:?}"))
	}
}

fn add_triangle_fan<'a, T: 'a, I>(
	dst_vertices: &mut Vec<T>,
	dst_indices: &mut Vec<u16>,
	vertices: I,
) where
	I: Iterator<Item = &'a T>,
	T: Copy,
{
	let first_index = dst_vertices.len();
	dst_vertices.extend(vertices);
	let len = dst_vertices.len() - first_index;
	dst_indices.reserve((len - 2) * 3);
	for i in (first_index + 1)..(first_index + len - 1) {
		dst_indices.push(first_index as u16);
		dst_indices.push(i as u16);
		dst_indices.push((i + 1) as u16);
	}
}

fn add_rect(
	vertices: &mut Vec<VertexPos2Coord2RGBA<f32>>,
	indices: &mut Vec<u16>,
	bounds: Aabr<f32>,
	texture_coordinate_bounds: Aabr<f32>,
	color: Rgba<f32>,
) {
	add_triangle_fan(
		vertices,
		indices,
		vec![
			VertexPos2Coord2RGBA {
				pos: bounds.min,
				coord: texture_coordinate_bounds.min,
				color,
			},
			VertexPos2Coord2RGBA {
				pos: Vec2::new(bounds.min.x, bounds.max.y),
				coord: Vec2::new(
					texture_coordinate_bounds.min.x,
					texture_coordinate_bounds.max.y,
				),
				color,
			},
			VertexPos2Coord2RGBA {
				pos: bounds.max,
				coord: texture_coordinate_bounds.max,
				color,
			},
			VertexPos2Coord2RGBA {
				pos: Vec2::new(bounds.max.x, bounds.min.y),
				coord: Vec2::new(
					texture_coordinate_bounds.max.x,
					texture_coordinate_bounds.min.y,
				),
				color,
			},
		]
		.iter(),
	);
}
