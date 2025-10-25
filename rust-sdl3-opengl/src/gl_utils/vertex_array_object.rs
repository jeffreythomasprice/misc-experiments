use std::ffi::c_void;

use color_eyre::eyre::Result;

use crate::gl_utils::{
    buffer::Buffer,
    shader::{Shader, ShaderAttribute, ShaderAttributeType, ShaderProgram},
};

pub struct VertexArrayObject {
    instance: u32,
}

impl VertexArrayObject {
    pub fn new_array_and_element_array_buffers<T>(
        shader: &ShaderProgram,
        array_buffer: &Buffer<T>,
        element_array_buffer: &Buffer<u16>,
        attributes: &[(&str, usize)],
    ) -> Result<Self> {
        unsafe {
            let mut instance = 0;
            gl::GenVertexArrays(1, &mut instance);
            gl::BindVertexArray(instance);
            for attribute in shader.attributes.iter() {
                gl::EnableVertexAttribArray(attribute.location);
            }
            array_buffer.bind();
            element_array_buffer.bind();
            for (name, offset) in attributes.iter() {
                vertex_attrib_pointer::<T>(shader.assert_attribute_by_name(name)?, *offset);
            }
            Ok(Self { instance })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.instance);
        }
    }
}

fn vertex_attrib_pointer<T>(attribute: &ShaderAttribute, offset: usize) {
    let (size, gl_type) = match attribute.typ {
        ShaderAttributeType::Float => (1, gl::FLOAT),
        ShaderAttributeType::FloatVec2 => (2, gl::FLOAT),
        ShaderAttributeType::FloatVec3 => (3, gl::FLOAT),
        ShaderAttributeType::FloatVec4 => (4, gl::FLOAT),
    };
    let size = size * attribute.size;
    unsafe {
        gl::VertexAttribPointer(
            attribute.location,
            size,
            gl_type,
            gl::FALSE,
            size_of::<T>() as i32,
            offset as *const c_void,
        );
    }
}
