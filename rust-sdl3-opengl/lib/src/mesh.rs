use std::rc::Rc;

use bytemuck::Pod;
use color_eyre::eyre::Result;

use crate::gl_utils::{
    buffer::{Buffer, BufferTarget, BufferUsage},
    shader::ShaderProgram,
    vertex_array_object::{VertexArrayObject, VertexAttributeDefinition},
};

pub struct Mesh<Vertex> {
    shader: Rc<ShaderProgram>,
    attributes: Vec<VertexAttributeDefinition>,
    array_buffer: Buffer<Vertex>,
    element_array_buffer: Buffer<u16>,
    vertex_array_object: VertexArrayObject,
}

// TODO more conveniences to make it easy to append vertices to a mesh, mesh builder

impl<Vertex> Mesh<Vertex>
where
    Vertex: Pod,
{
    pub fn new(
        shader: Rc<ShaderProgram>,
        attributes: Vec<VertexAttributeDefinition>,
        vertices: &[Vertex],
        indices: &[u16],
    ) -> Result<Self> {
        let array_buffer = Buffer::new(BufferTarget::Array, BufferUsage::DynamicDraw, vertices)?;
        let element_array_buffer = Buffer::new(
            BufferTarget::ElementArray,
            BufferUsage::DynamicDraw,
            indices,
        )?;
        let vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
            &shader,
            &array_buffer,
            &element_array_buffer,
            &attributes,
        )?;
        Ok(Self {
            shader,
            attributes,
            array_buffer,
            element_array_buffer,
            vertex_array_object,
        })
    }

    pub fn bind(&self) {
        self.vertex_array_object.bind();
    }

    pub fn array_buffer(&self) -> &Buffer<Vertex> {
        &self.array_buffer
    }

    pub fn element_array_buffer(&self) -> &Buffer<u16> {
        &self.element_array_buffer
    }

    pub fn buffers_mut<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Buffer<Vertex>, &mut Buffer<u16>) -> Result<()>,
    {
        f(&mut self.array_buffer, &mut self.element_array_buffer)?;
        self.recreate_vertex_array_object()?;
        Ok(())
    }

    fn recreate_vertex_array_object(&mut self) -> Result<()> {
        self.vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
            &self.shader,
            &self.array_buffer,
            &self.element_array_buffer,
            &self.attributes,
        )?;
        Ok(())
    }
}
