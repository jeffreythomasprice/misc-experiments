use std::{marker::PhantomData, rc::Rc};

use bytemuck::Pod;
use lib::{
    error::Error,
    graphics::{
        buffer::Buffer,
        shader::{AttributePointer, ShaderProgram, TypedUniform},
        texture::Texture,
    },
};
use nalgebra::Matrix4;
use web_sys::WebGl2RenderingContext;

use crate::mesh::Mesh;

pub enum DrawMode {
    Triangles,
    // TODO triangle fan, etc.
}

impl DrawMode {
    pub fn gl_constant(&self) -> u32 {
        match self {
            DrawMode::Triangles => WebGl2RenderingContext::TRIANGLES,
        }
    }
}

pub struct RenderPhase<Vertex>
where
    Vertex: Pod,
{
    context: Rc<WebGl2RenderingContext>,
    shader: ShaderProgram,
    attributes: Vec<AttributePointer>,
    sampler_uniform: Option<TypedUniform<i32>>,
    projection_matrix_uniform: Option<TypedUniform<Matrix4<f32>>>,
    model_view_matrix_uniform: Option<TypedUniform<Matrix4<f32>>>,
    _phantom: PhantomData<Vertex>,
}

pub struct Renderer<'a, Vertex> {
    context: &'a WebGl2RenderingContext,
    attributes: &'a [AttributePointer],
    _phantom: PhantomData<Vertex>,
}

impl<Vertex> RenderPhase<Vertex>
where
    Vertex: Pod,
{
    pub fn new(
        context: Rc<WebGl2RenderingContext>,
        shader: ShaderProgram,
        // TODO prove that the attributes come from this shader?
        attributes: Vec<AttributePointer>,
        sampler_uniform_name: Option<&str>,
        projection_matrix_uniform_name: Option<&str>,
        model_view_matrix_uniform_name: Option<&str>,
    ) -> Result<Self, Error> {
        let sampler_uniform = match sampler_uniform_name {
            Some(name) => Some(shader.get_uniform_by_name(name)?.clone().into()),
            None => None,
        };

        let projection_matrix_uniform = match projection_matrix_uniform_name {
            Some(name) => Some(shader.get_uniform_by_name(name)?.clone().into()),
            None => None,
        };

        let model_view_matrix_uniform = match model_view_matrix_uniform_name {
            Some(name) => Some(shader.get_uniform_by_name(name)?.clone().into()),
            None => None,
        };

        Ok(Self {
            context,
            shader,
            attributes,
            sampler_uniform,
            projection_matrix_uniform,
            model_view_matrix_uniform,
            _phantom: PhantomData {},
        })
    }

    pub fn perform_batch<F>(
        &self,
        texture: Option<&Texture>,
        projection_matrix: Option<&Matrix4<f32>>,
        model_view_matrix: Option<&Matrix4<f32>>,
        mut f: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&Renderer<Vertex>) -> Result<(), Error>,
    {
        self.shader.use_program();

        if let Some(m) = &projection_matrix {
            if let Some(uniform) = &self.projection_matrix_uniform {
                uniform.set(m, false);
            } else {
                Err("projection matrix is provided, but shader doesn't support it")?;
            }
        }

        if let Some(m) = model_view_matrix {
            if let Some(uniform) = &self.model_view_matrix_uniform {
                uniform.set(m, false);
            } else {
                Err("model view matrix is provided, but shader doesn't support it")?;
            }
        }

        if let Some(t) = texture {
            if let Some(uniform) = &self.sampler_uniform {
                uniform.set(0);
                self.context.active_texture(WebGl2RenderingContext::TEXTURE0);
                t.bind();
            } else {
                Err("texture is provided but shader doesn't support it")?;
            }
        }

        let err = f(&Renderer {
            context: &self.context,
            attributes: &self.attributes,
            _phantom: PhantomData {},
        });

        if let Some(t) = texture {
            t.bind_none();
        }

        self.shader.use_none();

        err
    }
}

impl<Vertex> Renderer<'_, Vertex> {
    // TODO draw arrays

    pub fn draw_elements(&self, array_buffer: &Buffer<Vertex>, element_array_buffer: &Buffer<u16>, draw_mode: DrawMode)
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
}

impl<Vertex> Renderer<'_, Vertex>
where
    Vertex: Pod,
{
    pub fn draw_mesh(&self, mesh: &mut Mesh<Vertex>, draw_mode: DrawMode) -> Result<(), Error> {
        let (array_buffer, element_array_buffer) = mesh.buffers();
        self.draw_elements(array_buffer.buffer_mut()?, element_array_buffer.buffer_mut()?, draw_mode);
        Ok(())
    }
}
