use crate::events::{
    self, EventHandler, MouseButton, MouseMoveEvent, MousePressEvent, NextEventHandler, State,
};
use crate::geometry::{camera::Camera, size::Size};
use crate::graphics::buffer_usage::BufferUsage;
use crate::graphics::element_array_buffer::ElementArrayBuffer;
use crate::graphics::{self};
use crate::graphics::{
    array_buffer::ArrayBuffer,
    shader::{AttributePointer, ShaderProgram},
};
use anyhow::{anyhow, Result};
use bytemuck::{offset_of, Pod, Zeroable};
use nalgebra::{Isometry3, Point3};
use nalgebra_glm::{vec4, Vec2, Vec3, Vec4};
use std::{sync::Arc, time::Duration};
use web_sys::WebGl2RenderingContext;

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec2,
    ray_origin: Point3<f32>,
    ray_delta: Vec3,
}

pub struct DemoState {
    state: State,
    context: Arc<WebGl2RenderingContext>,

    shader: ShaderProgram,
    position_attribute: AttributePointer,
    ray_origin_attribute: AttributePointer,
    ray_delta_attribute: AttributePointer,

    array_buffer: ArrayBuffer<Vertex>,
    element_array_buffer: ElementArrayBuffer,

    camera: Camera,
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

        let ray_origin_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("ray_origin_attribute")
                .ok_or(anyhow!("failed to find ray origin attribute"))?
                .clone(),
            3,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, ray_origin) as i32,
        );

        let ray_delta_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("ray_delta_attribute")
                .ok_or(anyhow!("failed to find ray delta attribute"))?
                .clone(),
            3,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, ray_delta) as i32,
        );

        let array_buffer = ArrayBuffer::new_with_len(context.clone(), BufferUsage::DynamicDraw, 4)?;

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
            ray_origin_attribute,
            ray_delta_attribute,

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

        // make a new set of rays based on the size of the camera
        // assume a quad at the origin, with the eye of the camera on one side looking at the quad
        // the rays will go from the eye to the four corners
        // the height of the quad will be fixed to 1, and the width set such that the quad has the same aspect ratio as the screen
        // the distance from the quad to the eye will be such that it matches the camera's field of view
        // let camera_quad_height = 1.0f32;
        // let camera_quad_width =
        let camera_quad_size = Size {
            width: 2.0f32 * (self.camera.get_screen_size().width as f32)
                / (self.camera.get_screen_size().height as f32),
            height: 2.0f32,
        };
        let camera_distance =
            camera_quad_size.width * 0.5f32 / (self.camera.field_of_view() * 0.5f32).tan();
        let ray_origin = Point3::new(0.0, 0.0, camera_distance);
        let mut array_buffer_data = [
            Vertex {
                position: Vec2::new(-1.0, -1.0),
                ray_origin,
                ray_delta: (Point3::new(
                    -camera_quad_size.width * 0.5,
                    -camera_quad_size.height * 0.5,
                    0.0,
                ) - ray_origin)
                    .normalize(),
            },
            Vertex {
                position: Vec2::new(1.0, -1.0),
                ray_origin,
                ray_delta: (Point3::new(
                    camera_quad_size.width * 0.5,
                    -camera_quad_size.height * 0.5,
                    0.0,
                ) - ray_origin)
                    .normalize(),
            },
            Vertex {
                position: Vec2::new(1.0, 1.0),
                ray_origin,
                ray_delta: (Point3::new(
                    camera_quad_size.width * 0.5,
                    camera_quad_size.height * 0.5,
                    0.0,
                ) - ray_origin)
                    .normalize(),
            },
            Vertex {
                position: Vec2::new(-1.0, 1.0),
                ray_origin,
                ray_delta: (Point3::new(
                    -camera_quad_size.width * 0.5,
                    camera_quad_size.height * 0.5,
                    0.0,
                ) - ray_origin)
                    .normalize(),
            },
        ];
        // transform the rays by the camera matrix
        let model_view_inverse = self
            .camera
            .model_view_matrix()
            .try_inverse()
            .ok_or(anyhow!("camera modelview matrix is not invertable"))?;
        for vertex in array_buffer_data.iter_mut() {
            vertex.ray_origin = model_view_inverse.transform_point(&vertex.ray_origin);
            vertex.ray_delta = model_view_inverse.transform_vector(&vertex.ray_delta);
        }
        // actually update the array buffer with new rays
        self.array_buffer.set(&array_buffer_data, 0)?;

        self.shader.use_program();

        self.array_buffer.bind();
        self.element_array_buffer.bind();

        self.position_attribute.enable();
        self.ray_origin_attribute.enable();
        self.ray_delta_attribute.enable();
        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.element_array_buffer.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );
        self.position_attribute.disable();
        self.ray_origin_attribute.disable();
        self.ray_delta_attribute.disable();

        self.element_array_buffer.bind_none();
        self.array_buffer.bind_none();
        self.shader.use_none();

        Ok(NextEventHandler::NoChange)
    }

    fn update(&mut self, delta: Duration) -> Result<events::NextEventHandler> {
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
