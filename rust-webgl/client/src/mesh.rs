use std::rc::Rc;

use bytemuck::Pod;
use lib::{
    error::Error,
    graphics::{buffer::BufferUsage, vec_buffer::VecBuffer},
};
use web_sys::WebGl2RenderingContext;

pub struct Mesh<Vertex>
where
    Vertex: Pod,
{
    array_buffer: VecBuffer<Vertex>,
    element_array_buffer: VecBuffer<u16>,
}

impl<Vertex> Mesh<Vertex>
where
    Vertex: Pod,
{
    pub fn new(context: Rc<WebGl2RenderingContext>) -> Result<Self, Error> {
        Ok(Self {
            array_buffer: VecBuffer::new_array_buffer(context.clone(), BufferUsage::DynamicDraw)?,
            element_array_buffer: VecBuffer::new_element_array_buffer(context.clone(), BufferUsage::DynamicDraw)?,
        })
    }

    pub fn buffers(&mut self) -> (&mut VecBuffer<Vertex>, &mut VecBuffer<u16>) {
        (&mut self.array_buffer, &mut self.element_array_buffer)
    }

    pub fn clear(&mut self) {
        self.array_buffer.clear();
        self.element_array_buffer.clear();
    }

    pub fn push_triangle(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        let i = self.array_buffer.len() as u16;
        self.array_buffer.extend_from_slice(&[a, b, c]);
        self.element_array_buffer.extend_from_slice(&[i, i + 1, i + 2]);
    }

    pub fn push_triangle_fan(&mut self, vertices: &[Vertex]) {
        let i = self.array_buffer.len() as u16;
        self.array_buffer.extend_from_slice(vertices);
        for j in (i + 1)..(i + (vertices.len() as u16) - 1) {
            let k = j + 1;
            self.element_array_buffer.extend_from_slice(&[i, j, k]);
        }
    }
}
