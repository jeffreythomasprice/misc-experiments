mod mesh;
mod render_phase;

use bytemuck::{Pod, Zeroable};
use gloo::utils::document;
use lib::{
    error::Error,
    events::{KeyPressEvent, MouseButton, MouseMoveEvent, MousePressEvent},
    graphics::{
        self,
        colors::{F32RGBA, U8RGBA},
        shader::{AttributePointer, AttributePointerType, ShaderProgram},
        texture::Texture,
        texture_font::TextureFont,
    },
    math::{camera::Camera, size::Size, vec2::Vec2, vec3::Vec3},
    uistate::{run, UIState},
};
use log::*;
use mesh::Mesh;
use nalgebra::Matrix4;
use nalgebra_glm::rotate_y;
use render_phase::DrawMode;
use render_phase::RenderPhase;
use std::{collections::HashMap, f32::consts::TAU, io::Cursor, mem::offset_of, panic, rc::Rc, sync::Mutex, time::Duration};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex2 {
    position: Vec2<f32>,
    texture_coordinate: Vec2<f32>,
    color: F32RGBA,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex3 {
    position: Vec3<f32>,
    texture_coordinate: Vec2<f32>,
    color: F32RGBA,
}

struct State {
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<WebGl2RenderingContext>,

    key_state: Rc<Mutex<HashMap<String, bool>>>,

    render_phase_3d: RenderPhase<Vertex3>,
    render_phase_2d: RenderPhase<Vertex2>,

    image_texture: Texture,
    image_texture_mesh: Mesh<Vertex3>,

    procedural_texture: Texture,
    procedural_texture_mesh: Mesh<Vertex2>,

    font: TextureFont<'static>,
    font_mesh: Mesh<Vertex2>,

    camera: Camera,
    ortho_matrix: Matrix4<f32>,

    rotation: f32,
}

impl State {
    pub async fn new(canvas: Rc<HtmlCanvasElement>, context: Rc<WebGl2RenderingContext>) -> Result<State, Error> {
        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader_3d.vertex.glsl"),
            include_str!("shaders/shader_3d.fragment.glsl"),
        )?;
        let attributes = vec![
            AttributePointer::new::<Vertex3>(
                shader.get_attribute_by_name("position_attribute")?.clone(),
                3,
                AttributePointerType::Float,
                false,
                offset_of!(Vertex3, position) as i32,
            ),
            AttributePointer::new::<Vertex3>(
                shader.get_attribute_by_name("texture_coordinate_attribute")?.clone(),
                2,
                graphics::shader::AttributePointerType::Float,
                false,
                offset_of!(Vertex3, texture_coordinate) as i32,
            ),
            AttributePointer::new::<Vertex3>(
                shader.get_attribute_by_name("color_attribute")?.clone(),
                4,
                graphics::shader::AttributePointerType::Float,
                false,
                offset_of!(Vertex3, color) as i32,
            ),
        ];
        let render_phase_3d = RenderPhase::new(
            context.clone(),
            shader,
            attributes,
            Some("sampler_uniform"),
            Some("projection_matrix_uniform"),
            Some("model_view_matrix_uniform"),
        )?;

        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader_2d.vertex.glsl"),
            include_str!("shaders/shader_2d.fragment.glsl"),
        )?;
        let attributes = vec![
            AttributePointer::new::<Vertex2>(
                shader.get_attribute_by_name("position_attribute")?.clone(),
                2,
                AttributePointerType::Float,
                false,
                offset_of!(Vertex2, position) as i32,
            ),
            AttributePointer::new::<Vertex2>(
                shader.get_attribute_by_name("texture_coordinate_attribute")?.clone(),
                2,
                graphics::shader::AttributePointerType::Float,
                false,
                offset_of!(Vertex2, texture_coordinate) as i32,
            ),
            AttributePointer::new::<Vertex2>(
                shader.get_attribute_by_name("color_attribute")?.clone(),
                4,
                graphics::shader::AttributePointerType::Float,
                false,
                offset_of!(Vertex2, color) as i32,
            ),
        ];
        let render_phase_2d = RenderPhase::new(
            context.clone(),
            shader,
            attributes,
            Some("sampler_uniform"),
            Some("projection_matrix_uniform"),
            Some("model_view_matrix_uniform"),
        )?;

        let image_texture = {
            let image_bytes = include_bytes!("../assets/vader.jpg");
            Texture::new_with_image_data(context.clone(), Some("vader.jpg"), Cursor::new(image_bytes))?
        };

        let mut static_texture_mesh = Mesh::new(context.clone())?;
        {
            let height = (image_texture.size().height as f32) / (image_texture.size().width as f32);
            static_texture_mesh.push_triangle_fan(&[
                Vertex3 {
                    position: Vec3 {
                        x: -1.0,
                        y: -height,
                        z: 0.0,
                    },
                    texture_coordinate: Vec2 { x: 0.0, y: 1.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex3 {
                    position: Vec3 {
                        x: 1.0,
                        y: -height,
                        z: 0.0,
                    },
                    texture_coordinate: Vec2 { x: 1.0, y: 1.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex3 {
                    position: Vec3 { x: 1.0, y: height, z: 0.0 },
                    texture_coordinate: Vec2 { x: 1.0, y: 0.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex3 {
                    position: Vec3 {
                        x: -1.0,
                        y: height,
                        z: 0.0,
                    },
                    texture_coordinate: Vec2 { x: 0.0, y: 0.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ]);
        }

        let (procedural_texture, procedural_texture_mesh) = {
            let size = Size { width: 256, height: 256 };

            let mut pixels = Vec::with_capacity(size.width * size.height);
            for y in 0..size.height {
                let b = ((y as f64) / ((size.height - 1) as f64) * 255.0) as u8;
                for x in 0..size.width {
                    let a = ((x as f64) / ((size.width - 1) as f64) * 255.0) as u8;
                    pixels.push(U8RGBA {
                        red: a,
                        green: b,
                        blue: 255 - a,
                        alpha: 255,
                    });
                }
            }

            let size = Size {
                width: size.width as u32,
                height: size.height as u32,
            };

            let mut texture = Texture::new_with_pixels(
                context.clone(),
                size,
                &(0..(size.width * size.height))
                    .map(|_| U8RGBA {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    })
                    .collect::<Vec<_>>(),
            )?;
            texture.copy_pixels(
                &lib::math::vec2::Vec2 { x: 64, y: 64 },
                &lib::math::rect::Rect::with_position_and_size(lib::math::vec2::Vec2 { x: 64, y: 128 }, Size { width: 512, height: 512 }),
                &size,
                &pixels,
            )?;
            texture.copy_pixels(
                &lib::math::vec2::Vec2 { x: 128, y: 192 },
                &lib::math::rect::Rect::with_position_and_size(lib::math::vec2::Vec2 { x: 64, y: 64 }, Size { width: 128, height: 128 }),
                &size,
                &pixels,
            )?;

            let mut mesh = Mesh::new(context.clone())?;
            mesh.push_triangle_fan(&[
                Vertex2 {
                    position: Vec2 { x: 0.0, y: 0.0 },
                    texture_coordinate: Vec2 { x: 0.0, y: 0.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex2 {
                    position: Vec2 {
                        x: size.width as f32,
                        y: 0.0,
                    },
                    texture_coordinate: Vec2 { x: 1.0, y: 0.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex2 {
                    position: Vec2 {
                        x: size.width as f32,
                        y: size.height as f32,
                    },
                    texture_coordinate: Vec2 { x: 1.0, y: 1.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex2 {
                    position: Vec2 {
                        x: 0.0,
                        y: size.height as f32,
                    },
                    texture_coordinate: Vec2 { x: 0.0, y: 1.0 },
                    color: F32RGBA {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ]);

            (texture, mesh)
        };

        let font = TextureFont::new_with_bytes_and_scale(context.clone(), include_bytes!("../assets/Ubuntu/Ubuntu-Regular.ttf"), 30.0)?;

        let font_mesh = Mesh::new(context.clone())?;

        Ok(State {
            canvas,
            context,

            key_state: Rc::new(Mutex::new(HashMap::new())),

            render_phase_3d,
            render_phase_2d,

            image_texture,
            image_texture_mesh: static_texture_mesh,

            procedural_texture,
            procedural_texture_mesh,

            font,
            font_mesh,

            camera: Camera::new(
                60.0f32.to_radians(),
                Size { width: 0, height: 0 },
                1.0,
                1000.0,
                Vec3 { x: 0.0, y: 0.0, z: 6.0 },
                Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                Vec3 { x: 0.0, y: 1.0, z: 0.0 },
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
    fn resize(&mut self, size: Size<u32>) -> Result<(), Error> {
        self.context.viewport(0, 0, size.width as i32, size.height as i32);

        self.camera.set_screen_size(size);

        self.ortho_matrix = Matrix4::new_orthographic(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);

        Ok(())
    }

    fn render(&mut self) -> Result<(), Error> {
        self.context.clear_color(149.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.render_phase_3d.perform_batch(
            Some(&self.image_texture),
            Some(self.camera.projection_matrix()),
            Some(&rotate_y(self.camera.model_view_matrix(), self.rotation)),
            |renderer| renderer.draw_mesh(&mut self.image_texture_mesh, DrawMode::Triangles),
        )?;

        self.render_phase_2d.perform_batch(
            Some(&self.procedural_texture),
            Some(&self.ortho_matrix),
            Some(&Matrix4::identity()),
            |renderer| renderer.draw_mesh(&mut self.procedural_texture_mesh, DrawMode::Triangles),
        )?;

        // TODO simplify font api? combine layout, update_cache, and rect_for all in one?
        if let Some(layout) = self.font.layout("Hello, World!") {
            self.font.update_cache(layout.glyphs.iter())?;

            self.font_mesh.clear();
            for glyph in layout.glyphs.iter() {
                if let Some((uv_rect, screen_rect)) = self.font.rect_for(glyph)? {
                    self.font_mesh.push_triangle_fan(&[
                        Vertex2 {
                            position: Vec2 {
                                x: screen_rect.min.x as f32,
                                y: screen_rect.min.y as f32,
                            },
                            texture_coordinate: Vec2 {
                                x: uv_rect.min.x,
                                y: uv_rect.min.y,
                            },
                            color: F32RGBA {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 1.0,
                            },
                        },
                        Vertex2 {
                            position: Vec2 {
                                x: screen_rect.max.x as f32,
                                y: screen_rect.min.y as f32,
                            },
                            texture_coordinate: Vec2 {
                                x: uv_rect.max.x,
                                y: uv_rect.min.y,
                            },
                            color: F32RGBA {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 1.0,
                            },
                        },
                        Vertex2 {
                            position: Vec2 {
                                x: screen_rect.max.x as f32,
                                y: screen_rect.max.y as f32,
                            },
                            texture_coordinate: Vec2 {
                                x: uv_rect.max.x,
                                y: uv_rect.max.y,
                            },
                            color: F32RGBA {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 1.0,
                            },
                        },
                        Vertex2 {
                            position: Vec2 {
                                x: screen_rect.min.x as f32,
                                y: screen_rect.max.y as f32,
                            },
                            texture_coordinate: Vec2 {
                                x: uv_rect.min.x,
                                y: uv_rect.max.y,
                            },
                            color: F32RGBA {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 1.0,
                            },
                        },
                    ]);
                };
            }

            self.context.enable(WebGl2RenderingContext::BLEND);
            self.context
                .blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);

            self.render_phase_2d.perform_batch(
                Some(self.font.get_texture()),
                Some(&self.ortho_matrix),
                Some(&Matrix4::identity()),
                |renderer| renderer.draw_mesh(&mut self.font_mesh, DrawMode::Triangles),
            )?;

            self.context.disable(WebGl2RenderingContext::BLEND);
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
