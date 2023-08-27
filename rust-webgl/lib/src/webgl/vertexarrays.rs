use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use crate::errors::Result;

use super::{buffers::Buffer, shaders::ShaderAttributeInfo};

pub struct VertexArrayAttribute<'a> {
    pub shader_attribute: &'a ShaderAttributeInfo,
    pub buffer: &'a Buffer,
    pub size: i32,
    pub type_: u32,
    pub normalized: bool,
    pub stride: usize,
    pub offset: usize,
}

pub struct VertexArray {
    context: Rc<WebGl2RenderingContext>,
    vertex_array: WebGlVertexArrayObject,
}

impl VertexArray {
    pub fn new(
        context: Rc<WebGl2RenderingContext>,
        attributes: &[VertexArrayAttribute],
    ) -> Result<VertexArray> {
        let vertex_array = context
            .create_vertex_array()
            .ok_or("failed to create vetex array")?;
        context.bind_vertex_array(Some(&vertex_array));

        for attr in attributes {
            attr.buffer.bind();
            context.enable_vertex_attrib_array(attr.shader_attribute.location as u32);
            context.vertex_attrib_pointer_with_i32(
                attr.shader_attribute.location as u32,
                attr.size,
                attr.type_,
                attr.normalized,
                attr.stride as i32,
                attr.offset as i32,
            );
        }

        context.bind_vertex_array(None);
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        Ok(VertexArray {
            context,
            vertex_array,
        })
    }

    pub fn bind(&self) {
        self.context.bind_vertex_array(Some(&self.vertex_array));
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        self.context.delete_vertex_array(Some(&self.vertex_array))
    }
}
