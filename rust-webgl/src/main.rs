mod app;
mod errors;
mod extra_math;
mod shaders;

use std::{collections::HashMap, panic, rc::Rc};

use app::{App, AppContext, EventHandler, MouseEvent};
use errors::JsInteropError;
use extra_math::LookAtCamera;
use js_sys::Uint8Array;
use log::*;
use nalgebra::{Matrix4, Point2, Point3, Unit, Vector2, Vector3};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{app::document, shaders::ShaderProgram};

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
    element_array_buffer: WebGlBuffer,
    vertex_array: WebGlVertexArrayObject,

    key_state: HashMap<u32, bool>,

    rotation: f32,

    perspective_transform: Matrix4<f32>,
    camera: LookAtCamera,
}

impl DemoState {
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        *self.key_state.get(&key_code).unwrap_or(&false)
    }
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
        buffer_data_with_slice(
            &gl,
            WebGl2RenderingContext::ARRAY_BUFFER,
            &[
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
                    position: Vector3::new(1.0, 1.0, 0.0),
                    color: Rgba {
                        r: 0.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                },
                Vertex {
                    position: Vector3::new(-1.0, 1.0, 0.0),
                    color: Rgba {
                        r: 0.5,
                        g: 0.0,
                        b: 0.5,
                        a: 1.0,
                    },
                },
            ],
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        let element_array_buffer = gl.create_buffer().ok_or(JsInteropError::NotFound(
            "failed to create element buffer".to_owned(),
        ))?;
        gl.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&element_array_buffer),
        );
        buffer_data_with_slice::<u16>(
            &gl,
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &[0, 1, 2, 2, 3, 0],
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);

        let vertex_array = gl.create_vertex_array().ok_or(JsInteropError::NotFound(
            "failed to create vertex array object".to_owned(),
        ))?;
        gl.bind_vertex_array(Some(&vertex_array));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&array_buffer));
        enable_vertex_attribute_and_set_pointer(
            &gl,
            &shader_program,
            "positionAttribute",
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            core::mem::size_of::<Vertex>() as i32,
            core::mem::offset_of!(Vertex, position) as i32,
        )?;
        enable_vertex_attribute_and_set_pointer(
            &gl,
            &shader_program,
            "colorAttribute",
            4,
            WebGl2RenderingContext::FLOAT,
            false,
            core::mem::size_of::<Vertex>() as i32,
            core::mem::offset_of!(Vertex, color) as i32,
        )?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        Ok(Self {
            shader_program,
            element_array_buffer,
            vertex_array,

            key_state: HashMap::new(),

            rotation: 0f32,

            perspective_transform: Matrix4::identity(),
            camera: LookAtCamera::new(
                Point3::new(0.0, 0.0, -6.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ),
        })
    }

    fn handle_event(&mut self, event: app::Event) -> Result<(), DemoError> {
        match event {
            app::Event::Resize {
                context,
                width,
                height,
            } => {
                let gl = context.gl;

                gl.viewport(0, 0, width as i32, height as i32);

                self.perspective_transform = Matrix4::new_perspective(
                    width as f32 / height as f32,
                    60.0f32.to_radians(),
                    1.0,
                    100.0,
                );

                Ok(())
            }

            app::Event::MouseDown(_) => Ok(()),

            app::Event::MouseUp(e) => {
                if document()?.pointer_lock_element() == Some(e.context.canvas.clone().into()) {
                    document()?.exit_pointer_lock();
                } else {
                    e.context.canvas.request_pointer_lock();
                }

                Ok(())
            }

            app::Event::MouseMove(e) => {
                let desired = Point2::from(
                    Vector2::new(e.context.canvas.width(), e.context.canvas.height()) / 2,
                );
                if document()?.pointer_lock_element() == Some(e.context.canvas.clone().into())
                    && e.point() != desired
                {
                    self.camera
                        .turn(Vector2::new(-e.delta().x as f32, -e.delta().y as f32));
                }

                Ok(())
            }

            app::Event::KeyDown(e) => {
                self.key_state.insert(e.event.key_code(), true);
                Ok(())
            }

            app::Event::KeyUp(e) => {
                self.key_state.insert(e.event.key_code(), false);
                Ok(())
            }

            app::Event::Render { context } => {
                let gl = context.gl;

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
                    (self.camera.transform_matrix()
                        * Matrix4::from_axis_angle(
                            &Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
                            self.rotation,
                        ))
                    .as_slice(),
                );

                gl.bind_vertex_array(Some(&self.vertex_array));
                gl.bind_buffer(
                    WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                    Some(&self.element_array_buffer),
                );
                gl.draw_elements_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    6,
                    WebGl2RenderingContext::UNSIGNED_SHORT,
                    0,
                );
                gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
                gl.bind_vertex_array(None);

                gl.use_program(None);

                Ok(())
            }

            app::Event::Update { context: _, delta } => {
                self.rotation +=
                    (delta.as_secs_f32() * 90.0f32.to_radians()) % 360.0f32.to_radians();

                const UP: u32 = 38;
                const LEFT: u32 = 37;
                const DOWN: u32 = 40;
                const RIGHT: u32 = 39;
                const W: u32 = 87;
                const A: u32 = 65;
                const S: u32 = 83;
                const D: u32 = 68;
                const SHIFT: u32 = 16;
                const SPACE: u32 = 32;
                let forward = if self.is_key_pressed(UP) || self.is_key_pressed(W) {
                    1f32
                } else {
                    0f32
                } + if self.is_key_pressed(DOWN) || self.is_key_pressed(S) {
                    -1f32
                } else {
                    0f32
                };
                let strafe = if self.is_key_pressed(RIGHT) || self.is_key_pressed(D) {
                    1f32
                } else {
                    0f32
                } + if self.is_key_pressed(LEFT) || self.is_key_pressed(A) {
                    -1f32
                } else {
                    0f32
                };
                let up = if self.is_key_pressed(SPACE) {
                    1f32
                } else {
                    0f32
                } + if self.is_key_pressed(SHIFT) {
                    -1f32
                } else {
                    0f32
                };
                const CAMERA_SPEED: f32 = 3.5f32;
                let camera_delta = CAMERA_SPEED * delta.as_secs_f32();
                self.camera.move_position(
                    forward * camera_delta,
                    strafe * camera_delta,
                    up * camera_delta,
                );

                Ok(())
            }
        }
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

// TODO JEFF buffers.rs?

fn enable_vertex_attribute_and_set_pointer(
    gl: &WebGl2RenderingContext,
    shader: &ShaderProgram,
    attr_name: &str,
    size: i32,
    type_: u32,
    normalized: bool,
    stride: i32,
    offset: i32,
) -> Result<(), JsInteropError> {
    let attr = shader
        .get_attribute_by_name(attr_name)
        .ok_or(JsInteropError::NotFound(
            "failed to find attribute".to_owned(),
        ))?;
    gl.vertex_attrib_pointer_with_i32(attr.index, size, type_, normalized, stride, offset);
    gl.enable_vertex_attrib_array(attr.index);
    Ok(())
}

fn buffer_data_with_slice<T>(gl: &WebGl2RenderingContext, target: u32, src_data: &[T], usage: u32) {
    unsafe {
        gl.buffer_data_with_array_buffer_view(
            target,
            &Uint8Array::view_mut_raw(
                src_data.as_ptr() as *mut u8,
                core::mem::size_of::<T>() * src_data.len(),
            ),
            usage,
        );
    }
}
