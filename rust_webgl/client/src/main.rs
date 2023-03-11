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
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
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

		gl.blend_func(
			WebGl2RenderingContext::SRC_ALPHA,
			WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
		);
		gl.enable(WebGl2RenderingContext::BLEND);

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

		gl.disable(WebGl2RenderingContext::BLEND);

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

				// let texture =
				// 	Texture2d::new_with_image_url(gl.clone(), "assets/bricks.png").await?;
				let texture = Texture2d::new_with_canvas(gl.clone(), &new_test_image()?)?;

				let data = Rc::new(RefCell::new(Data::new(shader, texture)?));
				let next_state = Rc::new(RefCell::new(DemoState::new(data)));
				Ok::<AppStateHandle, AppError>(next_state)
			},
		))))
	}) {
		error!(format!("fatal {e:?}"))
	}
}

// TODO JEFF rename me, should take some generating params
fn new_test_image() -> Result<HtmlCanvasElement, AppError> {
	new_canvas_image(Extent2::new(800, 800), |context, size| {
		let bounds = Rect::new(0, 0, size.w, size.h)
			.as_::<f64, f64>()
			.into_aabr();

		let checkerboard = new_checkerboard_image(
			Extent2::new(size.w / 4, size.h / 4),
			Extent2::new(size.w as f64 / 8f64, size.h as f64 / 8f64),
			&"#ffa552".into(),
			&"#212354".into(),
		)?;
		let pattern = context
			.create_pattern_with_html_canvas_element(&checkerboard, "repeat")?
			.ok_or("failed to make pattern")?;

		context.clear_rect(0f64, 0f64, size.w as f64, size.h as f64);

		let corner_radius = if bounds.size().w < bounds.size().h {
			bounds.size().w
		} else {
			bounds.size().h
		} * 0.28f64;
		let rounded_rect_bounds = {
			let inner_size = bounds.size() * 0.97f64;
			let offset = (bounds.size() - inner_size) * 0.5f64;
			Aabr {
				min: bounds.min + offset,
				max: bounds.max - offset,
			}
		};
		rounded_rect(context, rounded_rect_bounds, corner_radius)?;
		context.set_fill_style(&pattern);
		context.fill();
		context.set_stroke_style(&"black".into());
		context.set_line_width(5f64);
		context.stroke();

		Ok(())
	})
}

fn new_checkerboard_image(
	size: Extent2<u32>,
	tile_size: Extent2<f64>,
	fill_style_0: &JsValue,
	fill_style_1: &JsValue,
) -> Result<HtmlCanvasElement, AppError> {
	new_canvas_image(size, |context, size| {
		context.clear_rect(0f64, 0f64, size.w as f64, size.h as f64);

		let mut x = 0f64;
		let mut first_style_index_on_row = 0;
		while x < size.w as f64 {
			let mut y = 0f64;
			let mut style_index = first_style_index_on_row;
			while y < size.h as f64 {
				context.set_fill_style(if style_index == 0 {
					fill_style_0
				} else {
					fill_style_1
				});
				context.fill_rect(x, y, tile_size.w, tile_size.h);
				y += tile_size.h;
				style_index = (style_index + 1) % 2;
			}
			x += tile_size.w;
			first_style_index_on_row = (first_style_index_on_row + 1) % 2;
		}

		Ok(())
	})
}

fn rounded_rect(
	context: &CanvasRenderingContext2d,
	rect: Aabr<f64>,
	corner_radius: f64,
) -> Result<(), AppError> {
	context.begin_path();
	context.move_to(rect.min.x, rect.min.x + corner_radius);
	context.arc_to(
		rect.min.x,
		rect.min.y,
		rect.min.x + corner_radius,
		rect.min.y,
		corner_radius,
	)?;
	context.line_to(rect.max.x - corner_radius, rect.min.y);
	context.arc_to(
		rect.max.x,
		rect.min.y,
		rect.max.x,
		rect.min.y + corner_radius,
		corner_radius,
	)?;
	context.line_to(rect.max.x, rect.max.y - corner_radius);
	context.arc_to(
		rect.max.x,
		rect.max.y,
		rect.max.x - corner_radius,
		rect.max.y,
		corner_radius,
	)?;
	context.line_to(rect.min.x + corner_radius, rect.max.y);
	context.arc_to(
		rect.min.x,
		rect.max.y,
		rect.min.x,
		rect.max.y - corner_radius,
		corner_radius,
	)?;
	context.close_path();
	Ok(())
}
