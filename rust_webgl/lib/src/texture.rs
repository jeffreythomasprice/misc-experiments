use std::rc::Rc;

use vek::Extent2;
use web_sys::{HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlTexture};

use crate::{new_image_from_url, AppError};

pub struct Texture2d {
	gl: Rc<WebGl2RenderingContext>,
	texture: WebGlTexture,
	size: Extent2<u32>,
}

impl Texture2d {
	pub fn new(gl: Rc<WebGl2RenderingContext>) -> Result<Self, AppError> {
		let gl = gl.clone();
		let texture = gl.create_texture().ok_or("failed to create texture")?;
		Ok(Self {
			gl,
			texture,
			size: Extent2::new(0, 0),
		})
	}

	pub fn new_with_canvas(
		gl: Rc<WebGl2RenderingContext>,
		source: &HtmlCanvasElement,
	) -> Result<Self, AppError> {
		let mut result = Self::new(gl)?;
		result.set_to_canvas(source)?;
		Ok(result)
	}

	pub fn new_with_image(
		gl: Rc<WebGl2RenderingContext>,
		source: &HtmlImageElement,
	) -> Result<Self, AppError> {
		let mut result = Self::new(gl)?;
		result.set_to_image(source)?;
		Ok(result)
	}

	pub async fn new_with_image_url(
		gl: Rc<WebGl2RenderingContext>,
		url: &str,
	) -> Result<Self, AppError> {
		let mut result = Self::new(gl)?;
		result.set_to_image_url(url).await?;
		Ok(result)
	}

	pub fn size(&self) -> Extent2<u32> {
		self.size
	}

	pub fn bind(&self) {
		self.gl
			.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture))
	}

	pub fn bind_none(&self) {
		self.gl
			.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None)
	}

	pub fn set_to_canvas(&mut self, source: &HtmlCanvasElement) -> Result<(), AppError> {
		let gl = self.gl.clone();
		self.bind();
		gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element(
			WebGl2RenderingContext::TEXTURE_2D,
			0,
			WebGl2RenderingContext::RGBA.try_into().unwrap(),
			WebGl2RenderingContext::RGBA,
			WebGl2RenderingContext::UNSIGNED_BYTE,
			source,
		)?;
		self.set_props(Extent2::new(source.width(), source.height()));
		self.bind_none();
		Ok(())
	}

	pub fn set_to_image(&mut self, source: &HtmlImageElement) -> Result<(), AppError> {
		let gl = self.gl.clone();
		self.bind();
		gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
			WebGl2RenderingContext::TEXTURE_2D,
			0,
			WebGl2RenderingContext::RGBA.try_into().unwrap(),
			WebGl2RenderingContext::RGBA,
			WebGl2RenderingContext::UNSIGNED_BYTE,
			source,
		)?;
		self.set_props(Extent2::new(source.width(), source.height()));
		self.bind_none();
		Ok(())
	}

	pub async fn set_to_image_url(&mut self, url: &str) -> Result<(), AppError> {
		let source = new_image_from_url(url).await?;
		self.set_to_image(&source)?;
		Ok(())
	}

	fn set_props(&mut self, size: Extent2<u32>) {
		// TODO different props if it's a power of 2 size
		self.size = size;
		let gl = self.gl.clone();
		gl.tex_parameteri(
			WebGl2RenderingContext::TEXTURE_2D,
			WebGl2RenderingContext::TEXTURE_MAG_FILTER,
			WebGl2RenderingContext::LINEAR as i32,
		);
		gl.tex_parameteri(
			WebGl2RenderingContext::TEXTURE_2D,
			WebGl2RenderingContext::TEXTURE_MIN_FILTER,
			WebGl2RenderingContext::LINEAR as i32,
		);
		gl.tex_parameteri(
			WebGl2RenderingContext::TEXTURE_2D,
			WebGl2RenderingContext::TEXTURE_WRAP_S,
			WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
		);
		gl.tex_parameteri(
			WebGl2RenderingContext::TEXTURE_2D,
			WebGl2RenderingContext::TEXTURE_WRAP_T,
			WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
		);
	}
}

impl Drop for Texture2d {
	fn drop(&mut self) {
		self.gl.delete_texture(Some(&self.texture))
	}
}
