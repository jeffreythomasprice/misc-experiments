use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::AppError;

pub struct Texture {
	gl: Rc<WebGl2RenderingContext>,
	texture: WebGlTexture,
}

impl Texture {
	pub fn new(gl: Rc<WebGl2RenderingContext>) -> Result<Self, AppError> {
		let gl = gl.clone();
		let texture = gl.create_texture().ok_or("failed to create texture")?;
		Ok(Self { gl, texture })
	}

	pub fn bind(&self) {
		self.gl
			.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture))
	}

	pub fn bind_none(&self) {
		self.gl
			.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None)
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		self.gl.delete_texture(Some(&self.texture))
	}
}
