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
        texture_font::TextureFont,
    },
    math::{camera::Camera, rect::Rect, size::Size},
    uistate::{run, UIState},
};
use log::*;
use nalgebra::{Matrix4, Vector2};
use nalgebra_glm::{rotate_y, DVec2, U32Vec2, Vec2, Vec3};
use rusttype::{gpu_cache::Cache, Font};
use std::{
    array,
    collections::HashMap,
    f32::consts::{PI, TAU},
    io::Cursor,
    mem::offset_of,
    panic,
    rc::Rc,
    sync::Mutex,
    time::Duration,
};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex2 {
    position: Vec2,
    texture_coordinate: Vec2,
    color: F32RGBA,
}

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex3 {
    position: Vec3,
    texture_coordinate: Vec2,
    color: F32RGBA,
}

struct State {
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<WebGl2RenderingContext>,

    key_state: Rc<Mutex<HashMap<String, bool>>>,

    shader_3d: ShaderProgram,
    position_attribute_3d: AttributePointer,
    texture_coordinate_attribute_3d: AttributePointer,
    color_attribute_3d: AttributePointer,
    sampler_uniform_3d: Uniform,
    projection_matrix_uniform_3d: Uniform,
    model_view_matrix_uniform_3d: Uniform,

    shader_2d: ShaderProgram,
    position_attribute_2d: AttributePointer,
    texture_coordinate_attribute_2d: AttributePointer,
    color_attribute_2d: AttributePointer,
    sampler_uniform_2d: Uniform,
    projection_matrix_uniform_2d: Uniform,
    model_view_matrix_uniform_2d: Uniform,

    static_texture_array_buffer: ArrayBuffer<Vertex3>,
    static_texture_element_aray_buffer: ElementArrayBuffer,
    texture: Texture,

    font_array_buffer: ArrayBuffer<Vertex2>,
    font_element_array_buffer: ElementArrayBuffer,
    font: TextureFont<'static>,

    camera: Camera,
    ortho_matrix: Matrix4<f32>,

    rotation: f32,
}

impl State {
    pub async fn new(canvas: Rc<HtmlCanvasElement>, context: Rc<WebGl2RenderingContext>) -> Result<State, Error> {
        let shader_3d = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader_3d.vertex.glsl"),
            include_str!("shaders/shader_3d.fragment.glsl"),
        )?;

        let position_attribute_3d = AttributePointer::new::<Vertex3>(
            shader_3d.assert_attribute_by_name("position_attribute")?.clone(),
            3,
            AttributePointerType::Float,
            false,
            offset_of!(Vertex3, position) as i32,
        );

        let texture_coordinate_attribute_3d = AttributePointer::new::<Vertex3>(
            shader_3d.assert_attribute_by_name("texture_coordinate_attribute")?.clone(),
            2,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex3, texture_coordinate) as i32,
        );

        let color_attribute_3d = AttributePointer::new::<Vertex3>(
            shader_3d.assert_attribute_by_name("color_attribute")?.clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex3, color) as i32,
        );

        let sampler_uniform_3d = shader_3d.assert_uniform_by_name("sampler_uniform")?.clone();

        let projection_matrix_uniform_3d = shader_3d.assert_uniform_by_name("projection_matrix_uniform")?.clone();

        let model_view_matrix_uniform_3d = shader_3d.assert_uniform_by_name("model_view_matrix_uniform")?.clone();

        let shader_2d = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader_2d.vertex.glsl"),
            include_str!("shaders/shader_2d.fragment.glsl"),
        )?;

        let position_attribute_2d = AttributePointer::new::<Vertex2>(
            shader_2d.assert_attribute_by_name("position_attribute")?.clone(),
            2,
            AttributePointerType::Float,
            false,
            offset_of!(Vertex3, position) as i32,
        );

        let texture_coordinate_attribute_2d = AttributePointer::new::<Vertex2>(
            shader_2d.assert_attribute_by_name("texture_coordinate_attribute")?.clone(),
            2,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex3, texture_coordinate) as i32,
        );

        let color_attribute_2d = AttributePointer::new::<Vertex2>(
            shader_2d.assert_attribute_by_name("color_attribute")?.clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex3, color) as i32,
        );

        let sampler_uniform_2d = shader_2d.assert_uniform_by_name("sampler_uniform")?.clone();

        let projection_matrix_uniform_2d = shader_2d.assert_uniform_by_name("projection_matrix_uniform")?.clone();

        let model_view_matrix_uniform_2d = shader_2d.assert_uniform_by_name("model_view_matrix_uniform")?.clone();

        let texture = {
            let image_bytes = include_bytes!("../assets/vader.jpg");
            Texture::new_with_image_data(context.clone(), Some("vader.jpg"), Cursor::new(image_bytes))?
        };

        let texture_aspect_ratio = (texture.size().height as f32) / (texture.size().width as f32);

        let static_texture_array_buffer = ArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[
                Vertex3 {
                    position: Vec3::new(-1.0, -texture_aspect_ratio, 0.0),
                    texture_coordinate: Vec2::new(0.0, 1.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex3 {
                    position: Vec3::new(1.0, -texture_aspect_ratio, 0.0),
                    texture_coordinate: Vec2::new(1.0, 1.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex3 {
                    position: Vec3::new(1.0, texture_aspect_ratio, 0.0),
                    texture_coordinate: Vec2::new(1.0, 0.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex3 {
                    position: Vec3::new(-1.0, texture_aspect_ratio, 0.0),
                    texture_coordinate: Vec2::new(0.0, 0.0),
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ],
        )?;

        let static_texture_element_aray_buffer =
            ElementArrayBuffer::new_with_data(context.clone(), BufferUsage::StaticDraw, &[0, 1, 2, 2, 3, 0])?;

        let font = TextureFont::new_with_bytes_and_scale(context.clone(), include_bytes!("../assets/Ubuntu/Ubuntu-Regular.ttf"), 30.0)?;

        let font_array_buffer = ArrayBuffer::new_with_len(context.clone(), BufferUsage::DynamicDraw, 0)?;
        let font_element_array_buffer = ElementArrayBuffer::new_with_len(context.clone(), BufferUsage::DynamicDraw, 0)?;

        Ok(State {
            canvas,
            context,

            key_state: Rc::new(Mutex::new(HashMap::new())),

            shader_3d,
            position_attribute_3d,
            texture_coordinate_attribute_3d,
            color_attribute_3d,
            sampler_uniform_3d,
            projection_matrix_uniform_3d,
            model_view_matrix_uniform_3d,

            shader_2d,
            position_attribute_2d,
            texture_coordinate_attribute_2d,
            color_attribute_2d,
            sampler_uniform_2d,
            projection_matrix_uniform_2d,
            model_view_matrix_uniform_2d,

            static_texture_array_buffer,
            static_texture_element_aray_buffer,
            texture,

            font,
            font_array_buffer,
            font_element_array_buffer,

            camera: Camera::new(
                60.0f32.to_radians(),
                Size { width: 0, height: 0 },
                1.0,
                1000.0,
                Vec3::new(0.0, 0.0, 6.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            ortho_matrix: Matrix4::identity(),

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

        self.camera.set_screen_size(Size { width, height });

        self.ortho_matrix = Matrix4::new_orthographic(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

        Ok(())
    }

    fn render(&mut self) -> Result<(), Error> {
        self.context.clear_color(149.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        {
            self.shader_3d.use_program();

            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.projection_matrix_uniform_3d.location),
                false,
                self.camera.projection_matrix().as_slice(),
            );
            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.model_view_matrix_uniform_3d.location),
                false,
                rotate_y(self.camera.model_view_matrix(), self.rotation).as_slice(),
            );

            self.context.uniform1i(Some(&self.sampler_uniform_3d.location), 0);
            self.context.active_texture(WebGl2RenderingContext::TEXTURE0);
            self.texture.bind();

            self.static_texture_array_buffer.bind();
            self.static_texture_element_aray_buffer.bind();

            self.position_attribute_3d.enable();
            self.texture_coordinate_attribute_3d.enable();
            self.color_attribute_3d.enable();

            self.context.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                self.static_texture_element_aray_buffer.len() as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );

            self.position_attribute_3d.disable();
            self.texture_coordinate_attribute_3d.disable();
            self.color_attribute_3d.disable();

            self.static_texture_array_buffer.bind_none();
            self.static_texture_element_aray_buffer.bind_none();

            self.texture.bind_none();

            self.shader_3d.use_none();
        }

        // TODO simplify font api? combine layout, update_cache, and rect_for all in one?
        if let Some(layout) = self.font.layout("Hello, World!") {
            self.font.update_cache(layout.glyphs.iter())?;

            // TODO convenience methods for resizing and setting data all at once?
            self.font_array_buffer.set_len(layout.glyphs.len() * 4);
            self.font_element_array_buffer.set_len(layout.glyphs.len() * 6);
            {
                let mut next_vertex: u16 = 0;
                let mut next_index: u16 = 0;
                for glyph in layout.glyphs.iter() {
                    if let Some((uv_rect, screen_rect)) = self.font.rect_for(glyph)? {
                        self.font_array_buffer.set(
                            &[
                                Vertex2 {
                                    position: Vec2::new(screen_rect.min.x as f32, screen_rect.min.y as f32),
                                    texture_coordinate: Vec2::new(uv_rect.min.x, uv_rect.min.y),
                                    color: F32RGBA {
                                        red: 1.0,
                                        green: 1.0,
                                        blue: 1.0,
                                        alpha: 1.0,
                                    },
                                },
                                Vertex2 {
                                    position: Vec2::new(screen_rect.max.x as f32, screen_rect.min.y as f32),
                                    texture_coordinate: Vec2::new(uv_rect.max.x, uv_rect.min.y),
                                    color: F32RGBA {
                                        red: 1.0,
                                        green: 1.0,
                                        blue: 1.0,
                                        alpha: 1.0,
                                    },
                                },
                                Vertex2 {
                                    position: Vec2::new(screen_rect.max.x as f32, screen_rect.max.y as f32),
                                    texture_coordinate: Vec2::new(uv_rect.max.x, uv_rect.max.y),
                                    color: F32RGBA {
                                        red: 1.0,
                                        green: 1.0,
                                        blue: 1.0,
                                        alpha: 1.0,
                                    },
                                },
                                Vertex2 {
                                    position: Vec2::new(screen_rect.min.x as f32, screen_rect.max.y as f32),
                                    texture_coordinate: Vec2::new(uv_rect.min.x, uv_rect.max.y),
                                    color: F32RGBA {
                                        red: 1.0,
                                        green: 1.0,
                                        blue: 1.0,
                                        alpha: 1.0,
                                    },
                                },
                            ],
                            next_vertex as usize,
                        )?;
                        self.font_element_array_buffer.set(
                            &[
                                next_vertex,
                                next_vertex + 1,
                                next_vertex + 2,
                                next_vertex + 2,
                                next_vertex + 3,
                                next_vertex,
                            ],
                            next_index as usize,
                        )?;
                        next_vertex += 4;
                        next_index += 6;
                    };
                }
            }

            // TODO blending

            self.shader_2d.use_program();

            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.projection_matrix_uniform_2d.location),
                false,
                self.ortho_matrix.as_slice(),
            );
            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.model_view_matrix_uniform_2d.location),
                false,
                Matrix4::identity().as_slice(),
            );

            self.context.uniform1i(Some(&self.sampler_uniform_2d.location), 0);
            self.context.active_texture(WebGl2RenderingContext::TEXTURE0);
            self.font.bind();

            self.font_array_buffer.bind();
            self.font_element_array_buffer.bind();

            self.position_attribute_2d.enable();
            self.texture_coordinate_attribute_2d.enable();
            self.color_attribute_2d.enable();

            self.context.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                self.font_element_array_buffer.len() as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );

            self.position_attribute_2d.disable();
            self.texture_coordinate_attribute_2d.disable();
            self.color_attribute_2d.disable();

            self.font_array_buffer.bind_none();
            self.font_element_array_buffer.bind_none();

            self.font.bind_none();

            self.shader_2d.use_none();
        }

        Ok(())
    }

    fn update(&mut self, delta: Duration) -> Result<(), Error> {
        let forward = if self.is_key_code_pressed("ArrowUp") || self.is_key_code_pressed("KeyW") {
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
        } - if self.is_key_code_pressed("ArrowLeft") || self.is_key_code_pressed("KeyA") {
            1.0f32
        } else {
            0.0f32
        };
        let up = if self.is_key_code_pressed("Space") { 1.0f32 } else { 0.0f32 }
            - if self.is_key_code_pressed("ShiftLeft") { 1.0f32 } else { 0.0f32 };
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
