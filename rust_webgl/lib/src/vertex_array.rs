use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use crate::app_states::AppError;

pub struct VertexArray {
	gl: Rc<WebGl2RenderingContext>,
	vertex_array: WebGlVertexArrayObject,
}

impl VertexArray {
	pub fn new(gl: Rc<WebGl2RenderingContext>) -> Result<Self, AppError> {
		let gl = gl.clone();
		let vertex_array = gl
			.create_vertex_array()
			.ok_or("failed to create vertex array")?;
		Ok(Self { gl, vertex_array })
	}

	pub fn bind(&self) {
		self.gl.bind_vertex_array(Some(&self.vertex_array));
	}

	pub fn bind_none(&self) {
		self.gl.bind_vertex_array(None);
	}
}

impl Drop for VertexArray {
	fn drop(&mut self) {
		self.gl.delete_vertex_array(Some(&self.vertex_array))
	}
}
