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
	mesh: Option<Mesh<VertexPos2Coord2RGBA<f32>>>,
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
			mesh: None,
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

		// TODO JEFF should be able to infer vertex type
		let mut mesh = Mesh::<VertexPos2Coord2RGBA<f32>>::new(
			gl.clone(),
			[
				VertexAttribute::new::<VertexPos2Coord2RGBA<f32>>(
					position_attribute,
					2,
					VertexAttributeDataType::Float,
					false,
					offset_of!(VertexPos2Coord2RGBA<f32>, pos),
				),
				VertexAttribute::new::<VertexPos2Coord2RGBA<f32>>(
					texture_coordinate_attribute,
					2,
					VertexAttributeDataType::Float,
					false,
					offset_of!(VertexPos2Coord2RGBA<f32>, coord),
				),
				VertexAttribute::new::<VertexPos2Coord2RGBA<f32>>(
					color_attribute,
					4,
					VertexAttributeDataType::Float,
					false,
					offset_of!(VertexPos2Coord2RGBA<f32>, color),
				),
			]
			// TODO JEFF should be able to do from_iter or whatever?
			.iter(),
		)?;
		{
			let size = self.data.borrow().texture.size();
			let color = Rgba::new(1f32, 1f32, 1f32, 1f32);
			mesh.add_triangle_fan(
				[
					VertexPos2Coord2RGBA {
						pos: Vec2::new(0f32, 0f32),
						coord: Vec2::new(0f32, 0f32),
						color,
					},
					VertexPos2Coord2RGBA {
						pos: Vec2::new(size.w as f32, 0f32),
						coord: Vec2::new(1f32, 0f32),
						color,
					},
					VertexPos2Coord2RGBA {
						pos: Vec2::new(size.w as f32, size.h as f32),
						coord: Vec2::new(1f32, 1f32),
						color,
					},
					VertexPos2Coord2RGBA {
						pos: Vec2::new(0f32, size.h as f32),
						coord: Vec2::new(0f32, 1f32),
						color,
					},
				]
				// TODO JEFF should be able to do from_iter or whatever?
				.iter(),
			);
			unsafe {
				mesh.flush();
			}
		}
		self.mesh = Some(mesh);

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

		if let (texture, Some(mesh)) = (&self.data.borrow().texture, &self.mesh) {
			texture.bind();

			mesh.bind();
			gl.draw_elements_with_i32(
				WebGl2RenderingContext::TRIANGLES,
				// TODO JEFF based on number of indices in mesh
				6,
				WebGl2RenderingContext::UNSIGNED_SHORT,
				0,
			);
			mesh.bind_none();

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
