use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::app_states::AppError;

pub enum BufferTarget {
	Array,
	ElementArray,
}

pub struct Buffer {
	gl: Rc<WebGl2RenderingContext>,
	gl_target: u32,
	buffer: WebGlBuffer,
}

impl Buffer {
	pub fn new(gl: Rc<WebGl2RenderingContext>, target: BufferTarget) -> Result<Self, AppError> {
		let gl = gl.clone();
		let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
		let gl_target = match target {
			BufferTarget::Array => WebGl2RenderingContext::ARRAY_BUFFER,
			BufferTarget::ElementArray => WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
		};
		Ok(Self {
			gl,
			gl_target,
			buffer,
		})
	}

	pub fn bind(&self) {
		self.gl.bind_buffer(self.gl_target, Some(&self.buffer));
	}

	pub fn bind_none(&self) {
		self.gl.bind_buffer(self.gl_target, None);
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		self.gl.delete_buffer(Some(&self.buffer))
	}
}
