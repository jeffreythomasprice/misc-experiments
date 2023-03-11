use std::rc::Rc;

use web_sys::WebGl2RenderingContext;

use crate::{AppError, AsUint8Array, AttributeInfo, Buffer, BufferTarget, VertexArray};

#[derive(Debug, Clone, Copy)]
pub enum VertexAttributeDataType {
	Float,
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
	attr: AttributeInfo,
	size: usize,
	gl_data_type: u32,
	normalized: bool,
	stride: usize,
	offset: usize,
}

impl VertexAttribute {
	pub fn new<T>(
		attr: &AttributeInfo,
		size: usize,
		data_type: VertexAttributeDataType,
		normalized: bool,
		offset: usize,
	) -> Self {
		Self {
			attr: attr.clone(),
			size,
			gl_data_type: match data_type {
				VertexAttributeDataType::Float => WebGl2RenderingContext::FLOAT,
			},
			normalized,
			stride: std::mem::size_of::<T>(),
			offset,
		}
	}
}

pub struct Mesh<T> {
	gl: Rc<WebGl2RenderingContext>,
	vertices: Vec<T>,
	indices: Vec<u16>,
	array_buffer: Buffer,
	element_array_buffer: Buffer,
	vertex_array: VertexArray,
}

impl<'a, T: 'a> Mesh<T>
where
	T: Copy,
{
	pub fn new<'b, I>(gl: Rc<WebGl2RenderingContext>, attributes: I) -> Result<Self, AppError>
	where
		I: Iterator<Item = &'b VertexAttribute>,
	{
		let vertices = Vec::new();
		let indices = Vec::new();
		let array_buffer = Buffer::new(gl.clone(), BufferTarget::Array)?;
		let element_array_buffer = Buffer::new(gl.clone(), BufferTarget::ElementArray)?;
		let vertex_array = VertexArray::new(gl.clone())?;

		vertex_array.bind();
		array_buffer.bind();
		element_array_buffer.bind();
		for attr in attributes {
			gl.enable_vertex_attrib_array(attr.attr.location);
			gl.vertex_attrib_pointer_with_i32(
				attr.attr.location,
				attr.size as i32,
				attr.gl_data_type,
				attr.normalized,
				attr.stride as i32,
				attr.offset as i32,
			);
		}
		vertex_array.bind_none();
		array_buffer.bind_none();
		element_array_buffer.bind_none();

		Ok(Self {
			gl: gl.clone(),
			vertices,
			indices,
			array_buffer,
			element_array_buffer,
			vertex_array,
		})
	}

	pub fn bind(&self) {
		self.vertex_array.bind()
	}

	pub fn bind_none(&self) {
		self.vertex_array.bind_none()
	}

	// TODO JEFF better meshes that track dirty regions and do subdata if possible
	pub unsafe fn flush(&self) {
		unsafe {
			self.array_buffer.bind();
			self.gl.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ARRAY_BUFFER,
				&self.vertices.as_uint8array(),
				WebGl2RenderingContext::STATIC_DRAW,
			);
			self.array_buffer.bind_none();

			self.element_array_buffer.bind();
			self.gl.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
				&self.indices.as_uint8array(),
				WebGl2RenderingContext::STATIC_DRAW,
			);
			self.element_array_buffer.bind_none();
		}
	}

	pub fn add_triangle_fan<'b, I>(&mut self, vertices: I)
	where
		I: Iterator<Item = &'b T>,
		'a: 'b,
	{
		let first_index = self.vertices.len();
		self.vertices.extend(vertices);
		let len = self.vertices.len() - first_index;
		self.indices.reserve((len - 2) * 3);
		for i in (first_index + 1)..(first_index + len - 1) {
			self.indices.push(first_index as u16);
			self.indices.push(i as u16);
			self.indices.push((i + 1) as u16);
		}
	}
}
