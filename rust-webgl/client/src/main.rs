use bytemuck::{Pod, Zeroable};
use gloo::utils::document;
use lib::{
    error::Error,
    events::{KeyPressEvent, MouseButton, MouseMoveEvent, MousePressEvent},
    graphics::{
        self,
        array_buffer::ArrayBuffer,
        buffer_usage::BufferUsage,
        colors::{F32RGBA, U8RGBA},
        element_array_buffer::ElementArrayBuffer,
        shader::{AttributePointer, AttributePointerType, ShaderProgram, Uniform},
        texture::Texture,
    },
    math::camera::Camera,
    uistate::{run, UIState},
};
use log::*;
use nalgebra::Vector2;
use nalgebra_glm::{rotate_y, DVec2, U32Vec2, Vec2, Vec3};
use std::{
    array,
    collections::HashMap,
    f32::consts::{PI, TAU},
    mem::offset_of,
    panic,
    rc::Rc,
    sync::Mutex,
    time::Duration,
};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    texture_coordinate: Vec2,
    color: F32RGBA,
}

struct State {
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<WebGl2RenderingContext>,

    key_state: Rc<Mutex<HashMap<String, bool>>>,

    shader: ShaderProgram,
    position_attribute: AttributePointer,
    texture_coordinate_attribute: AttributePointer,
    color_attribute: AttributePointer,
    sampler_uniform: Uniform,
    projection_matrix_uniform: Uniform,
    model_view_matrix_uniform: Uniform,

    array_buffer: ArrayBuffer<Vertex>,
    element_aray_buffer: ElementArrayBuffer,
    texture: Texture,

    camera: Camera,

    rotation: f32,
}

impl State {
    pub async fn new(
        canvas: Rc<HtmlCanvasElement>,
        context: Rc<WebGl2RenderingContext>,
    ) -> Result<State, Error> {
        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader.vertex.glsl"),
            include_str!("shaders/shader.fragment.glsl"),
        )?;

        let position_attribute = AttributePointer::new::<Vertex>(
            shader
                .assert_attribute_by_name("position_attribute")?
                .clone(),
            3,
            AttributePointerType::Float,
            false,
            offset_of!(Vertex, position) as i32,
        );

        let texture_coordinate_attribute = AttributePointer::new::<Vertex>(
            shader
                .assert_attribute_by_name("texture_coordinate_attribute")?
                .clone(),
            2,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, texture_coordinate) as i32,
        );

        let color_attribute = AttributePointer::new::<Vertex>(
            shader.assert_attribute_by_name("color_attribute")?.clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, color) as i32,
        );

        let sampler_uniform = shader.assert_uniform_by_name("sampler_uniform")?.clone();

        let projection_matrix_uniform = shader
            .assert_uniform_by_name("projection_matrix_uniform")?
            .clone();

        let model_view_matrix_uniform = shader
            .assert_uniform_by_name("model_view_matrix_uniform")?
            .clone();

        let array_buffer = ArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[
                Vertex {
                    position: Vec3::new(-1.0, -1.0, 0.0),
                    texture_coordinate: Vec2::new(0.0, 0.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(1.0, -1.0, 0.0),
                    texture_coordinate: Vec2::new(1.0, 0.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(1.0, 1.0, 0.0),
                    texture_coordinate: Vec2::new(1.0, 1.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(-1.0, 1.0, 0.0),
                    texture_coordinate: Vec2::new(0.0, 1.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ],
        )?;

        let element_aray_buffer = ElementArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[0, 1, 2, 2, 3, 0],
        )?;

        let texture = {
            let size = U32Vec2::new(256, 256);
            let mut pixels = Vec::with_capacity((size.x as usize) * (size.y as usize));
            for y in 0..size.y {
                let b = ((y as f64) * 255.0 / ((size.y - 1) as f64)) as u8;
                for x in 0..size.x {
                    let a = ((x as f64) * 255.0 / ((size.x - 1) as f64)) as u8;
                    pixels.push(U8RGBA {
                        red: a,
                        green: b,
                        blue: a,
                        alpha: 255,
                    });
                }
            }
            Texture::new_with_pixels(context.clone(), size, &pixels)?
        };

        Ok(State {
            canvas,
            context,

            key_state: Rc::new(Mutex::new(HashMap::new())),

            shader,
            position_attribute,
            texture_coordinate_attribute,
            color_attribute,
            sampler_uniform,
            projection_matrix_uniform,
            model_view_matrix_uniform,

            array_buffer,
            element_aray_buffer,
            texture,

            camera: Camera::new(
                60.0f32.to_radians(),
                Vector2::new(0, 0),
                1.0,
                1000.0,
                Vec3::new(0.0, 0.0, 6.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),

            rotation: 0.0f32,
        })
    }

    fn is_pointer_locked(&self) -> bool {
        match document().pointer_lock_element() {
            Some(canvas) if canvas == ***self.canvas => true,
            _ => false,
        }
    }

    fn set_pointer_lock(&self, b: bool) {
        if b {
            self.canvas.request_pointer_lock();
        } else {
            document().exit_pointer_lock();
        }
    }

    fn is_key_code_pressed(&self, code: &str) -> bool {
        let key_state = self.key_state.lock().unwrap();
        match key_state.get(code) {
            Some(true) => true,
            _ => false,
        }
    }
}

impl UIState for State {
    fn resize(&mut self, size: DVec2) -> Result<(), Error> {
        let width = size.x.floor() as u32;
        let height = size.y.floor() as u32;
        self.context.viewport(0, 0, width as i32, height as i32);
        self.camera.set_screen_size(Vector2::new(width, height));
        Ok(())
    }

    fn render(&mut self) -> Result<(), Error> {
        self.context
            .clear_color(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
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

        self.context
            .uniform1i(Some(&self.sampler_uniform.location), 0);
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.texture.bind();

        self.array_buffer.bind();
        self.element_aray_buffer.bind();

        self.position_attribute.enable();
        self.texture_coordinate_attribute.enable();
        self.color_attribute.enable();

        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.element_aray_buffer.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        self.position_attribute.disable();
        self.texture_coordinate_attribute.disable();
        self.color_attribute.disable();

        self.array_buffer.bind_none();
        self.element_aray_buffer.bind_none();

        self.texture.bind_none();

        self.shader.use_none();

        Ok(())
    }

    fn update(&mut self, delta: Duration) -> Result<(), Error> {
        let forward =
            if self.is_key_code_pressed("ArrowUp") || self.is_key_code_pressed("KeyW") {
                1.0f32
            } else {
                0.0f32
            } - if self.is_key_code_pressed("ArrowDown") || self.is_key_code_pressed("KeyS") {
                1.0f32
            } else {
                0.0f32
            };
        let right = if self.is_key_code_pressed("ArrowRight") || self.is_key_code_pressed("KeyD") {
            1.0f32
        } else {
            0.0f32
        } - if self.is_key_code_pressed("ArrowLeft") || self.is_key_code_pressed("KeyA")
        {
            1.0f32
        } else {
            0.0f32
        };
        let up = if self.is_key_code_pressed("Space") {
            1.0f32
        } else {
            0.0f32
        } - if self.is_key_code_pressed("ShiftLeft") {
            1.0f32
        } else {
            0.0f32
        };
        self.camera.move_based_on_current_axes(
            forward * 5.0f32 * delta.as_secs_f32(),
            up * 5.0f32 * delta.as_secs_f32(),
            right * 5.0f32 * delta.as_secs_f32(),
        );

        self.rotation = (self.rotation + delta.as_secs_f32() * 90.0f32.to_radians()) % TAU;

        Ok(())
    }

    fn mouse_up(&mut self, e: &MousePressEvent) -> Result<(), Error> {
        if let MouseButton::Left = e.button() {
            self.set_pointer_lock(!self.is_pointer_locked());
        }
        Ok(())
    }

    fn mouse_move(&mut self, e: &MouseMoveEvent) -> Result<(), Error> {
        if self.is_pointer_locked() {
            self.camera.turn_based_on_mouse_delta(e.delta());
        }
        Ok(())
    }

    fn key_down(&mut self, e: &KeyPressEvent) -> Result<(), Error> {
        let mut key_state = self.key_state.lock().unwrap();
        key_state.insert(e.code(), true);
        Ok(())
    }

    fn key_up(&mut self, e: &KeyPressEvent) -> Result<(), Error> {
        let mut key_state = self.key_state.lock().unwrap();
        key_state.insert(e.code(), false);
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).map_err(|e| e.to_string())?;

    run(|canvas, context| async {
        State::new(canvas, context).await.map(|state| {
            let result: Box<dyn UIState> = Box::new(state);
            result
        })
    })
}
