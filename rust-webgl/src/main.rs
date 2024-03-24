mod errors;
mod runner;
mod shaders;

use std::{panic, rc::Rc};

use errors::JsInteropError;
use js_sys::Uint8Array;
use log::*;
use nalgebra::{Matrix4, Unit, Vector3};
use runner::{App, EventHandler};
use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use crate::shaders::ShaderProgram;

#[allow(dead_code)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct Vertex {
    position: Vector3<f32>,
    color: Rgba,
}

struct DemoState {
    shader_program: ShaderProgram,
    vertex_array: WebGlVertexArrayObject,

    rotation: f32,

    perspective_transform: Matrix4<f32>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum DemoError {
    Js(JsInteropError),
}

impl From<JsInteropError> for DemoError {
    fn from(value: JsInteropError) -> Self {
        DemoError::Js(value)
    }
}

impl EventHandler<DemoError> for DemoState {
    fn init(gl: Rc<WebGl2RenderingContext>) -> Result<Self, DemoError> {
        let shader_program = ShaderProgram::new(
            gl.clone(),
            include_str!("shader.vert"),
            include_str!("shader.frag"),
        )?;

        let array_buffer = gl.create_buffer().ok_or(JsInteropError::NotFound(
            "failed to create buffer".to_owned(),
        ))?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&array_buffer));
        unsafe {
            let data = [
                Vertex {
                    position: Vector3::new(-1.0, -1.0, 0.0),
                    color: Rgba {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    },
                },
                Vertex {
                    position: Vector3::new(1.0, -1.0, 0.0),
                    color: Rgba {
                        r: 1.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    },
                },
                Vertex {
                    position: Vector3::new(0.0, 1.0, 0.0),
                    color: Rgba {
                        r: 0.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                },
            ];
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &Uint8Array::view_mut_raw(
                    data.as_ptr() as *mut u8,
                    core::mem::size_of::<Vertex>() * data.len(),
                ),
                WebGl2RenderingContext::STATIC_DRAW,
            );
        };
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        let vertex_array = gl.create_vertex_array().ok_or(JsInteropError::NotFound(
            "failed to create vertex array object".to_owned(),
        ))?;
        gl.bind_vertex_array(Some(&vertex_array));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&array_buffer));
        {
            let attr = shader_program
                .get_attribute_by_name("positionAttribute")
                .ok_or(JsInteropError::NotFound(
                    "failed to find attribute".to_owned(),
                ))?;
            gl.vertex_attrib_pointer_with_i32(
                attr.index,
                3,
                WebGl2RenderingContext::FLOAT,
                false,
                core::mem::size_of::<Vertex>() as i32,
                core::mem::offset_of!(Vertex, position) as i32,
            );
            gl.enable_vertex_attrib_array(attr.index);
        }
        {
            let attr = shader_program
                .get_attribute_by_name("colorAttribute")
                .ok_or(JsInteropError::NotFound(
                    "failed to find attribute".to_owned(),
                ))?;
            gl.vertex_attrib_pointer_with_i32(
                attr.index,
                4,
                WebGl2RenderingContext::FLOAT,
                false,
                core::mem::size_of::<Vertex>() as i32,
                core::mem::offset_of!(Vertex, color) as i32,
            );
            gl.enable_vertex_attrib_array(attr.index);
        }
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        Ok(Self {
            shader_program,
            vertex_array,

            rotation: 0f32,

            perspective_transform: Matrix4::identity(),
        })
    }

    fn resize(
        &mut self,
        gl: Rc<WebGl2RenderingContext>,
        width: f64,
        height: f64,
    ) -> Result<(), DemoError> {
        gl.viewport(0, 0, width as i32, height as i32);

        self.perspective_transform =
            Matrix4::new_perspective((width / height) as f32, 60.0f32.to_radians(), 1.0, 100.0);

        Ok(())
    }

    fn animate(
        &mut self,
        gl: Rc<WebGl2RenderingContext>,
        _total_time: f64,
        delta: std::time::Duration,
    ) -> Result<(), DemoError> {
        // cornflower blue, #6495ED
        gl.clear_color(0.39, 0.58, 0.93, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader_program.use_program();
        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &self
                    .shader_program
                    .get_uniform_by_name("projectionMatrix")
                    .ok_or(JsInteropError::NotFound(
                        "failed to find uniform".to_owned(),
                    ))?
                    .location,
            ),
            false,
            self.perspective_transform.as_slice(),
        );
        gl.uniform_matrix4fv_with_f32_array(
            Some(
                &self
                    .shader_program
                    .get_uniform_by_name("modelViewMatrix")
                    .ok_or(JsInteropError::NotFound(
                        "failed to find uniform".to_owned(),
                    ))?
                    .location,
            ),
            false,
            (Matrix4::new_translation(&Vector3::new(0.0, 0.0, -6.0))
                * Matrix4::from_axis_angle(
                    &Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
                    self.rotation,
                ))
            .as_slice(),
        );

        gl.bind_vertex_array(Some(&self.vertex_array));
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
        gl.bind_vertex_array(None);

        gl.use_program(None);

        self.rotation += (delta.as_secs_f32() * 90.0f32.to_radians()) % 360.0f32.to_radians();

        Ok(())
    }
}

fn main() -> Result<(), JsInteropError> {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    if let Err(e) = App::<DemoState>::run() {
        error!("app init error: {e:?}");
    }

    Ok(())
}
