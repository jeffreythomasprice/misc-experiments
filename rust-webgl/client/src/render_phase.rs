use std::rc::Rc;

use bytemuck::Pod;
use lib::{
    error::Error,
    graphics::{
        buffer::Buffer,
        shader::{AttributePointer, ShaderProgram, Uniform},
        texture::Texture,
    },
};
use nalgebra::Matrix4;
use web_sys::WebGl2RenderingContext;

use crate::draw_mode::DrawMode;

pub struct RenderPhase {
    context: Rc<WebGl2RenderingContext>,
    shader: ShaderProgram,
    attributes: Vec<AttributePointer>,
    sampler_uniform: Option<Uniform>,
    projection_matrix_uniform: Option<Uniform>,
    model_view_matrix_uniform: Option<Uniform>,
}

impl RenderPhase {
    pub fn new(
        context: Rc<WebGl2RenderingContext>,
        shader: ShaderProgram,
        // TODO prove that the attributes come from this shader?
        attributes: Vec<AttributePointer>,
        // TODO should take uniform names and then fail if they don't exist
        sampler_uniform: Option<Uniform>,
        projection_matrix_uniform: Option<Uniform>,
        model_view_matrix_uniform: Option<Uniform>,
    ) -> Self {
        Self {
            context,
            shader,
            attributes,
            sampler_uniform,
            projection_matrix_uniform,
            model_view_matrix_uniform,
        }
    }

    pub fn bind(
        &self,
        texture: Option<&Texture>,
        projection_matrix: Option<&Matrix4<f32>>,
        model_view_matrix: Option<&Matrix4<f32>>,
    ) -> Result<(), Error> {
        self.shader.use_program();

        if let Some(m) = &projection_matrix {
            if let Some(uniform) = &self.projection_matrix_uniform {
                self.context
                    .uniform_matrix4fv_with_f32_array(Some(&uniform.location), false, m.as_slice());
            } else {
                Err("projection matrix is provided, but shader doesn't support it")?;
            }
        }

        if let Some(m) = model_view_matrix {
            if let Some(uniform) = &self.model_view_matrix_uniform {
                self.context
                    .uniform_matrix4fv_with_f32_array(Some(&uniform.location), false, m.as_slice());
            } else {
                Err("model view matrix is provided, but shader doesn't support it")?;
            }
        }

        if let Some(t) = texture {
            if let Some(uniform) = &self.sampler_uniform {
                self.context.uniform1i(Some(&uniform.location), 0);
                self.context.active_texture(WebGl2RenderingContext::TEXTURE0);
                t.bind();
            } else {
                Err("texture is provided but shader doesn't support it")?;
            }
        }

        Ok(())
    }

    // TODO draw arrays

    pub fn draw_elements<Vertex>(&self, array_buffer: &Buffer<Vertex>, element_array_buffer: &Buffer<u16>, draw_mode: DrawMode)
    where
        Vertex: Pod,
    {
        array_buffer.bind();
        element_array_buffer.bind();

        for attr in self.attributes.iter() {
            attr.enable();
        }

        self.context.draw_elements_with_i32(
            draw_mode.gl_constant(),
            element_array_buffer.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        for attr in self.attributes.iter() {
            attr.disable();
        }

        array_buffer.bind_none();
        element_array_buffer.bind_none();
    }

    pub fn bind_none(&self) {
        // TODO unbind texture
        // if let Some(t) = &self.texture {
        //     t.bind_none();
        // }

        self.shader.use_none();
    }
}
