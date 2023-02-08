use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::AppError;

pub enum TextureTarget {
	Texture2d,
}

pub struct Texture {
	gl: Rc<WebGl2RenderingContext>,
	target: u32,
	texture: WebGlTexture,
}

impl Texture {
	pub fn new(gl: Rc<WebGl2RenderingContext>, target: TextureTarget) -> Result<Self, AppError> {
		let gl = gl.clone();
		let texture = gl.create_texture().ok_or("failed to create texture")?;
		Ok(Self {
			gl,
			target: match target {
				TextureTarget::Texture2d => WebGl2RenderingContext::TEXTURE_2D,
			},
			texture,
		})
	}

	pub fn target(&self) -> u32 {
		self.target
	}

	pub fn bind(&self) {
		self.gl.bind_texture(self.target, Some(&self.texture))
	}

	pub fn bind_none(&self) {
		self.gl.bind_texture(self.target, None)
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		self.gl.delete_texture(Some(&self.texture))
	}
}
