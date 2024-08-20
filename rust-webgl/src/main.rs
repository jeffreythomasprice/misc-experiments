mod dom;
mod events;
mod geometry;
mod graphics;

use anyhow::{anyhow, Result};
use bytemuck::{offset_of, Pod, Zeroable};
use events::{EventHandler, MouseButton, MouseMoveEvent, MousePressEvent, NextEventHandler, State};
use geometry::{camera::Camera, size::Size};
use graphics::{
    array_buffer::ArrayBuffer,
    shader::{AttributePointer, ShaderProgram, Uniform},
};
use log::*;
use nalgebra_glm::Vec3;
use std::{panic, sync::Arc, time::Duration};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct RGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    color: RGBA,
}

struct DemoState {
    state: State,
    context: Arc<WebGl2RenderingContext>,

    shader: ShaderProgram,
    position_attribute: AttributePointer,
    color_attribute: AttributePointer,
    projection_matrix_uniform: Uniform,
    model_view_matrix_uniform: Uniform,

    array_buffer: ArrayBuffer<Vertex>,

    camera: Camera,

    rotation: f32,
}

impl DemoState {
    pub fn new(state: State) -> Result<Self> {
        let context = state.context.clone();

        context.clear_color(0.25, 0.5, 1.0, 1.0);

        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("./demo/shaders/demo-vertex.glsl"),
            include_str!("./demo/shaders/demo-fragment.glsl"),
        )?;

        let position_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("position_attribute")
                .ok_or(anyhow!("failed to find position attribute"))?
                .clone(),
            2,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, position) as i32,
        );

        let color_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("color_attribute")
                .ok_or(anyhow!("failed to find color attribute"))?
                .clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, color) as i32,
        );

        let projection_matrix_uniform = shader
            .get_uniform_by_name("projection_matrix_uniform")
            .ok_or(anyhow!("failed to find projection matrix uniform"))?
            .clone();

        let model_view_matrix_uniform = shader
            .get_uniform_by_name("model_view_matrix_uniform")
            .ok_or(anyhow!("failed to find model view matrix uniform"))?
            .clone();

        let array_buffer = ArrayBuffer::new_with_data(
            context.clone(),
            graphics::array_buffer::Usage::DynamicDraw,
            &[
                Vertex {
                    position: Vec3::new(-1.0, -1.0, 0.0),
                    color: RGBA {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(1.0, -1.0, 0.0),
                    color: RGBA {
                        red: 0.0,
                        green: 1.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(0.0, 1.0, 0.0),
                    color: RGBA {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ],
        )?;

        Ok(Self {
            state,
            context,

            shader,
            position_attribute,
            color_attribute,
            projection_matrix_uniform,
            model_view_matrix_uniform,

            array_buffer,

            camera: Camera::new(
                60.0f32.to_radians(),
                Size {
                    width: 0,
                    height: 0,
                },
                1.0,
                1000.0,
                Vec3::new(0.0, 0.0, 6.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),

            rotation: 0.0,
        })
    }
}

impl EventHandler for DemoState {
    fn resize(&mut self, size: Size<u32>) -> Result<events::NextEventHandler> {
        self.context
            .viewport(0, 0, size.width as i32, size.height as i32);

        self.camera.set_screen_size(size);

        Ok(NextEventHandler::NoChange)
    }

    fn render(&mut self) -> Result<events::NextEventHandler> {
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader.use_program();

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.projection_matrix_uniform.location),
            false,
            self.camera.projection_matrix().as_slice(),
        );
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.model_view_matrix_uniform.location),
            false,
            self.camera.model_view_matrix().as_slice(),
        );
        // TODO apply rotation too

        self.array_buffer.bind();

        self.position_attribute.enable();
        self.color_attribute.enable();
        self.context.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            self.array_buffer.len() as i32,
        );
        self.position_attribute.disable();
        self.color_attribute.disable();

        self.array_buffer.bind_none();
        self.shader.use_none();

        Ok(NextEventHandler::NoChange)
    }

    fn update(&mut self, delta: Duration) -> Result<events::NextEventHandler> {
        self.rotation =
            (self.rotation + delta.as_secs_f32() * 90.0f32.to_radians()) % 360.0f32.to_radians();

        let forward = if self.state.is_key_code_pressed("ArrowUp")
            || self.state.is_key_code_pressed("KeyW")
        {
            1.0f32
        } else {
            0.0f32
        } - if self.state.is_key_code_pressed("ArrowDown")
            || self.state.is_key_code_pressed("KeyS")
        {
            1.0f32
        } else {
            0.0f32
        };
        let right = if self.state.is_key_code_pressed("ArrowRight")
            || self.state.is_key_code_pressed("KeyD")
        {
            1.0f32
        } else {
            0.0f32
        } - if self.state.is_key_code_pressed("ArrowLeft")
            || self.state.is_key_code_pressed("KeyA")
        {
            1.0f32
        } else {
            0.0f32
        };
        let up = if self.state.is_key_code_pressed("Space") {
            1.0f32
        } else {
            0.0f32
        } - if self.state.is_key_code_pressed("ShiftLeft") {
            1.0f32
        } else {
            0.0f32
        };
        self.camera.move_based_on_current_axes(
            forward * 5.0f32 * delta.as_secs_f32(),
            up * 5.0f32 * delta.as_secs_f32(),
            right * 5.0f32 * delta.as_secs_f32(),
        );

        Ok(NextEventHandler::NoChange)
    }

    fn mouse_up(&mut self, e: &MousePressEvent) -> Result<NextEventHandler> {
        if let MouseButton::Left = e.button() {
            self.state
                .set_pointer_lock(!self.state.is_pointer_locked()?)?;
        }
        Ok(NextEventHandler::NoChange)
    }

    fn mouse_move(&mut self, e: &MouseMoveEvent) -> Result<events::NextEventHandler> {
        if self.state.is_pointer_locked()? {
            self.camera.turn_based_on_mouse_delta(e.delta());
        }

        Ok(NextEventHandler::NoChange)
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        match events::run(Box::new(|context| match DemoState::new(context) {
            Ok(result) => Ok(Box::new(result)),
            Err(e) => Err(e),
        }))
        .await
        {
            Ok(_) => {
                warn!("state machine exited without error, but really it should keep going forever so something is probably wrong");
            }
            Err(e) => error!("state machine exited with: {e:?}"),
        };
    });
}
