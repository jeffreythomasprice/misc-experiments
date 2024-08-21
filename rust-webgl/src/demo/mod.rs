use crate::events::{
    self, EventHandler, MouseButton, MouseMoveEvent, MousePressEvent, NextEventHandler, State,
};
use crate::geometry::{camera::Camera, size::Size};
use crate::graphics::buffer_usage::BufferUsage;
use crate::graphics::element_array_buffer::ElementArrayBuffer;
use crate::graphics::{self, element_array_buffer};
use crate::graphics::{
    array_buffer::ArrayBuffer,
    shader::{AttributePointer, ShaderProgram, Uniform},
};
use anyhow::{anyhow, Result};
use bytemuck::{offset_of, Pod, Zeroable};
use nalgebra_glm::{rotate_y, Vec3};
use std::{sync::Arc, time::Duration};
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

pub struct DemoState {
    state: State,
    context: Arc<WebGl2RenderingContext>,

    shader: ShaderProgram,
    position_attribute: AttributePointer,
    color_attribute: AttributePointer,
    projection_matrix_uniform: Uniform,
    model_view_matrix_uniform: Uniform,

    array_buffer: ArrayBuffer<Vertex>,
    element_array_buffer: ElementArrayBuffer,

    camera: Camera,

    rotation: f32,
}

impl DemoState {
    pub fn new(state: State) -> Result<Self> {
        let context = state.context.clone();

        context.clear_color(0.25, 0.5, 1.0, 1.0);

        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("./shader-vertex.glsl"),
            include_str!("./shader-fragment.glsl"),
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
            BufferUsage::DynamicDraw,
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
                    position: Vec3::new(1.0, 1.0, 0.0),
                    color: RGBA {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(-1.0, 1.0, 0.0),
                    color: RGBA {
                        red: 1.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ],
        )?;

        let element_array_buffer = ElementArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::DynamicDraw,
            &[0, 1, 2, 2, 3, 0],
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
            element_array_buffer,

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
            rotate_y(self.camera.model_view_matrix(), self.rotation).as_slice(),
        );

        self.array_buffer.bind();
        self.element_array_buffer.bind();

        self.position_attribute.enable();
        self.color_attribute.enable();
        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.element_array_buffer.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );
        self.position_attribute.disable();
        self.color_attribute.disable();

        self.element_array_buffer.bind_none();
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
