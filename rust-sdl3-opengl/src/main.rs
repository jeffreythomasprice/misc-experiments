mod camera;
mod gl_utils;
mod sdl_utils;

use core::f32;
use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    sys::mouse::{SDL_HideCursor, SDL_ShowCursor, SDL_WarpMouseInWindow},
};

use crate::{
    camera::Camera,
    gl_utils::{
        buffer::{Buffer, BufferTarget, BufferUsage},
        shader::ShaderProgram,
        texture::Texture,
        vertex_array_object::VertexArrayObject,
    },
    sdl_utils::{AppState, sdl_main},
};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
struct Vertex {
    pub position: Vec3,
    _padding1: [u8; 4],
    pub texture_coordinate: Vec2,
    _padding2: [u8; 8],
    pub color: Vec4,
}

impl Vertex {
    pub fn new(position: Vec3, texture_coordinate: Vec2, color: Vec4) -> Self {
        Self {
            position,
            _padding1: Default::default(),
            texture_coordinate,
            _padding2: Default::default(),
            color,
        }
    }
}

struct App {
    texture: Texture,
    shader: ShaderProgram,
    array_buffer: Buffer<Vertex>,
    element_array_buffer: Buffer<u16>,
    vertex_array_object: VertexArrayObject,
    camera: Camera,
    is_mouse_locked: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let texture = Texture::new_image(load_image::load_data(include_bytes!(
            "../assets/bricks.png"
        ))?)?;

        let shader = ShaderProgram::new(
            include_str!("shaders/shader.vert"),
            include_str!("shaders/shader.frag"),
        )?;

        let color1 = vec4(1.0, 0.0, 0.0, 1.0);
        let color2 = vec4(0.0, 1.0, 0.0, 1.0);
        let color3 = vec4(0.0, 0.0, 1.0, 1.0);
        let color4 = vec4(1.0, 1.0, 0.0, 1.0);
        let color5 = vec4(0.0, 1.0, 1.0, 1.0);
        let color6 = vec4(1.0, 0.0, 1.0, 1.0);
        let array_buffer = Buffer::new(
            BufferTarget::Array,
            BufferUsage::StaticDraw,
            &[
                Vertex::new(vec3(-1.0, -1.0, -1.0), vec2(0.0, 0.0), color1),
                Vertex::new(vec3(1.0, -1.0, -1.0), vec2(1.0, 0.0), color1),
                Vertex::new(vec3(1.0, 1.0, -1.0), vec2(1.0, 1.0), color1),
                Vertex::new(vec3(-1.0, 1.0, -1.0), vec2(0.0, 1.0), color1),
                Vertex::new(vec3(-1.0, -1.0, 1.0), vec2(0.0, 0.0), color2),
                Vertex::new(vec3(1.0, -1.0, 1.0), vec2(1.0, 0.0), color2),
                Vertex::new(vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0), color2),
                Vertex::new(vec3(-1.0, 1.0, 1.0), vec2(0.0, 1.0), color2),
                Vertex::new(vec3(-1.0, -1.0, -1.0), vec2(0.0, 0.0), color3),
                Vertex::new(vec3(1.0, -1.0, -1.0), vec2(1.0, 0.0), color3),
                Vertex::new(vec3(1.0, -1.0, 1.0), vec2(1.0, 1.0), color3),
                Vertex::new(vec3(-1.0, -1.0, 1.0), vec2(0.0, 1.0), color3),
                Vertex::new(vec3(-1.0, 1.0, -1.0), vec2(0.0, 0.0), color4),
                Vertex::new(vec3(1.0, 1.0, -1.0), vec2(1.0, 0.0), color4),
                Vertex::new(vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0), color4),
                Vertex::new(vec3(-1.0, 1.0, 1.0), vec2(0.0, 1.0), color4),
                Vertex::new(vec3(-1.0, -1.0, -1.0), vec2(0.0, 0.0), color5),
                Vertex::new(vec3(-1.0, 1.0, -1.0), vec2(1.0, 0.0), color5),
                Vertex::new(vec3(-1.0, 1.0, 1.0), vec2(1.0, 1.0), color5),
                Vertex::new(vec3(-1.0, -1.0, 1.0), vec2(0.0, 1.0), color5),
                Vertex::new(vec3(1.0, -1.0, -1.0), vec2(0.0, 0.0), color6),
                Vertex::new(vec3(1.0, 1.0, -1.0), vec2(1.0, 0.0), color6),
                Vertex::new(vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0), color6),
                Vertex::new(vec3(1.0, -1.0, 1.0), vec2(0.0, 1.0), color6),
            ],
        )?;

        let element_array_buffer = Buffer::<u16>::new(
            BufferTarget::ElementArray,
            BufferUsage::StaticDraw,
            &[
                0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12,
                16, 17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
            ],
        )?;

        // TODO builder pattern?
        let vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
            &shader,
            &array_buffer,
            &element_array_buffer,
            &[
                ("in_position", bytemuck::offset_of!(Vertex, position)),
                (
                    "in_texture_coordinate",
                    bytemuck::offset_of!(Vertex, texture_coordinate),
                ),
                ("in_color", bytemuck::offset_of!(Vertex, color)),
            ],
        )?;

        let camera = Camera::new(
            vec3(0.0, 0.0, 6.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );

        Ok(Self {
            texture,
            shader,
            array_buffer,
            element_array_buffer,
            vertex_array_object,
            camera,
            is_mouse_locked: false,
        })
    }
}

impl sdl_utils::App for App {
    fn render(&self, AppState { window, .. }: &AppState) -> Result<()> {
        unsafe {
            gl::ClearColor(0.25, 0.5, 0.75, 1.0);
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::DEPTH_TEST);

            self.shader.use_program();

            let (width, height) = window.size();
            let ortho = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
            let perspective = Mat4::perspective_lh(
                f32::consts::FRAC_PI_6,
                (width as f32) / (height as f32),
                0.01,
                1000.0,
            );
            gl::UniformMatrix4fv(
                self.shader
                    .assert_uniform_by_name("uniform_projection_matrix")?
                    .location,
                1,
                gl::FALSE,
                perspective.to_cols_array().as_ptr(),
            );
            gl::UniformMatrix4fv(
                self.shader
                    .assert_uniform_by_name("uniform_modelview_matrix")?
                    .location,
                1,
                gl::FALSE,
                self.camera.matrix().to_cols_array().as_ptr(),
            );

            gl::ActiveTexture(gl::TEXTURE0);
            self.texture.bind();
            gl::Uniform1i(
                self.shader
                    .assert_uniform_by_name("uniform_sampler")?
                    .location,
                0,
            );

            self.vertex_array_object.bind();
            gl::DrawElements(
                gl::TRIANGLES,
                self.element_array_buffer.len() as i32,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }

        Ok(())
    }

    fn update(
        &mut self,
        AppState { keyboard, .. }: &AppState,
        elapsed_time: Duration,
    ) -> Result<()> {
        let elapsed_seconds = elapsed_time.as_secs_f32();
        let left = keyboard.is_pressed(Keycode::Left) || keyboard.is_pressed(Keycode::A);
        let right = keyboard.is_pressed(Keycode::Right) || keyboard.is_pressed(Keycode::D);
        let forward = keyboard.is_pressed(Keycode::Up) || keyboard.is_pressed(Keycode::W);
        let backward = keyboard.is_pressed(Keycode::Down) || keyboard.is_pressed(Keycode::S);
        let up = keyboard.is_pressed(Keycode::Space);
        let down = keyboard.is_pressed(Keycode::LShift) || keyboard.is_pressed(Keycode::RShift);
        let move_speed = elapsed_seconds * 10.0;
        self.camera.move_position(
            (if forward { 1.0 } else { 0.0 } - if backward { 1.0 } else { 0.0 }) * move_speed,
            (if right { 1.0 } else { 0.0 } - if left { 1.0 } else { 0.0 }) * move_speed,
            (if up { 1.0 } else { 0.0 } - if down { 1.0 } else { 0.0 }) * move_speed,
        );
        Ok(())
    }

    fn event(&mut self, AppState { window, .. }: &AppState, event: &Event) -> Result<()> {
        match event {
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.is_mouse_locked = !self.is_mouse_locked;
                unsafe {
                    if self.is_mouse_locked {
                        SDL_HideCursor();
                    } else {
                        SDL_ShowCursor();
                    }
                }
            }

            Event::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                if self.is_mouse_locked {
                    let (width, height) = window.size();
                    let center_x = (width as f32) / 2.0;
                    let center_y = (height as f32) / 2.0;

                    if !(*x == center_x && *y == center_y) {
                        self.camera.turn(vec2(*xrel, *yrel));
                    }

                    unsafe {
                        SDL_WarpMouseInWindow(window.raw(), center_x, center_y);
                    }
                }
            }

            _ => (),
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    sdl_main(App::new)?;
    Ok(())
}
