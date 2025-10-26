use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use core::f32;
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use sdl3::{
    event::Event,
    iostream::IOStream,
    keyboard::Keycode,
    mouse::MouseButton,
    sys::mouse::{SDL_HideCursor, SDL_ShowCursor, SDL_WarpMouseInWindow},
};
use std::{rc::Rc, time::Duration};
use tracing::*;

use lib::{
    camera::Camera,
    font::Font,
    gl_utils::{
        buffer::{Buffer, BufferTarget, BufferUsage},
        shader::ShaderProgram,
        texture::Texture,
        vertex_array_object::{VertexArrayObject, VertexAttributeDefinition},
    },
    sdl_utils::{AppState, sdl_main},
};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
struct Vertex2DColorTextureCoordinate {
    pub position: Vec2,
    pub texture_coordinate: Vec2,
    pub color: Vec4,
}

impl Vertex2DColorTextureCoordinate {
    pub fn new(position: Vec2, texture_coordinate: Vec2, color: Vec4) -> Self {
        Self {
            position,
            texture_coordinate,
            color,
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
struct Vertex3DColorTextureCoordinate {
    pub position: Vec3,
    _padding1: [u8; 4],
    pub texture_coordinate: Vec2,
    _padding2: [u8; 8],
    pub color: Vec4,
}

impl Vertex3DColorTextureCoordinate {
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
    font: Font,
    font_texture: Option<Texture>,
    shader2d: ShaderProgram,
    shader3d: ShaderProgram,
    // TODO meshes?
    font_array_buffer: Buffer<Vertex2DColorTextureCoordinate>,
    font_element_array_buffer: Buffer<u16>,
    font_vertex_array_object: VertexArrayObject,
    cube_array_buffer: Buffer<Vertex3DColorTextureCoordinate>,
    cube_element_array_buffer: Buffer<u16>,
    cube_vertex_array_object: VertexArrayObject,
    camera: Camera,
    is_mouse_locked: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let texture = Texture::new_image(load_image::load_data(include_bytes!(
            "../assets/bricks.png"
        ))?)?;

        let ttf_context = Rc::new(sdl3::ttf::init()?);
        let font = Font::new_from_vec(
            ttf_context,
            include_bytes!("../assets/IntelOneMono-Regular.ttf").into(),
            20.0,
        )?;

        let shader2d = ShaderProgram::new(
            include_str!("shaders/shader2d.vert"),
            include_str!("shaders/shader2d.frag"),
        )?;

        let shader3d = ShaderProgram::new(
            include_str!("shaders/shader3d.vert"),
            include_str!("shaders/shader3d.frag"),
        )?;

        let font_array_buffer = Buffer::new(
            BufferTarget::Array,
            BufferUsage::StreamDraw,
            &[Vertex2DColorTextureCoordinate::default(); 4],
        )?;
        let font_element_array_buffer = Buffer::new(
            BufferTarget::ElementArray,
            BufferUsage::StaticDraw,
            &[0, 1, 2, 2, 3, 0],
        )?;
        let font_vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
            &shader2d,
            &font_array_buffer,
            &font_element_array_buffer,
            &[
                VertexAttributeDefinition {
                    name: "in_position".to_owned(),
                    offset: bytemuck::offset_of!(Vertex2DColorTextureCoordinate, position),
                },
                VertexAttributeDefinition {
                    name: "in_texture_coordinate".to_owned(),
                    offset: bytemuck::offset_of!(
                        Vertex2DColorTextureCoordinate,
                        texture_coordinate
                    ),
                },
                VertexAttributeDefinition {
                    name: "in_color".to_owned(),
                    offset: bytemuck::offset_of!(Vertex2DColorTextureCoordinate, color),
                },
            ],
        )?;

        let color1 = vec4(1.0, 0.0, 0.0, 1.0);
        let color2 = vec4(0.0, 1.0, 0.0, 1.0);
        let color3 = vec4(0.0, 0.0, 1.0, 1.0);
        let color4 = vec4(1.0, 1.0, 0.0, 1.0);
        let color5 = vec4(0.0, 1.0, 1.0, 1.0);
        let color6 = vec4(1.0, 0.0, 1.0, 1.0);
        // TODO builder pattern?
        let cube_array_buffer = Buffer::new(
            BufferTarget::Array,
            BufferUsage::StaticDraw,
            &[
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, -1.0, -1.0), vec2(0.0, 0.0), color1),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, -1.0, -1.0), vec2(1.0, 0.0), color1),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, 1.0, -1.0), vec2(1.0, 1.0), color1),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, 1.0, -1.0), vec2(0.0, 1.0), color1),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, -1.0, 1.0), vec2(0.0, 0.0), color2),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, -1.0, 1.0), vec2(1.0, 0.0), color2),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0), color2),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, 1.0, 1.0), vec2(0.0, 1.0), color2),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, -1.0, -1.0), vec2(0.0, 0.0), color3),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, -1.0, -1.0), vec2(1.0, 0.0), color3),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, -1.0, 1.0), vec2(1.0, 1.0), color3),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, -1.0, 1.0), vec2(0.0, 1.0), color3),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, 1.0, -1.0), vec2(0.0, 0.0), color4),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, 1.0, -1.0), vec2(1.0, 0.0), color4),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0), color4),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, 1.0, 1.0), vec2(0.0, 1.0), color4),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, -1.0, -1.0), vec2(0.0, 0.0), color5),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, 1.0, -1.0), vec2(1.0, 0.0), color5),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, 1.0, 1.0), vec2(1.0, 1.0), color5),
                Vertex3DColorTextureCoordinate::new(vec3(-1.0, -1.0, 1.0), vec2(0.0, 1.0), color5),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, -1.0, -1.0), vec2(0.0, 0.0), color6),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, 1.0, -1.0), vec2(1.0, 0.0), color6),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0), color6),
                Vertex3DColorTextureCoordinate::new(vec3(1.0, -1.0, 1.0), vec2(0.0, 1.0), color6),
            ],
        )?;

        let cube_element_array_buffer = Buffer::<u16>::new(
            BufferTarget::ElementArray,
            BufferUsage::StaticDraw,
            &[
                0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12,
                16, 17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
            ],
        )?;

        // TODO builder pattern?
        let cube_vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
            &shader3d,
            &cube_array_buffer,
            &cube_element_array_buffer,
            &[
                VertexAttributeDefinition {
                    name: "in_position".to_owned(),
                    offset: bytemuck::offset_of!(Vertex3DColorTextureCoordinate, position),
                },
                VertexAttributeDefinition {
                    name: "in_texture_coordinate".to_owned(),
                    offset: bytemuck::offset_of!(
                        Vertex3DColorTextureCoordinate,
                        texture_coordinate
                    ),
                },
                VertexAttributeDefinition {
                    name: "in_color".to_owned(),
                    offset: bytemuck::offset_of!(Vertex3DColorTextureCoordinate, color),
                },
            ],
        )?;

        let camera = Camera::new(
            vec3(0.0, 0.0, 6.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );

        Ok(Self {
            texture,
            font,
            font_texture: None,
            shader2d,
            shader3d,
            font_array_buffer,
            font_element_array_buffer,
            font_vertex_array_object,
            cube_array_buffer,
            cube_element_array_buffer,
            cube_vertex_array_object,
            camera,
            is_mouse_locked: false,
        })
    }
}

impl lib::sdl_utils::App for App {
    fn render(&self, AppState { window, .. }: &AppState) -> Result<()> {
        let (width, height) = window.size();
        let ortho = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
        let perspective = Mat4::perspective_lh(
            f32::consts::FRAC_PI_6,
            (width as f32) / (height as f32),
            0.01,
            1000.0,
        );

        unsafe {
            gl::ClearColor(0.25, 0.5, 0.75, 1.0);
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::DEPTH_TEST);

            self.shader3d.use_program();

            gl::UniformMatrix4fv(
                self.shader3d
                    .assert_uniform_by_name("uniform_projection_matrix")?
                    .location,
                1,
                gl::FALSE,
                perspective.to_cols_array().as_ptr(),
            );
            gl::UniformMatrix4fv(
                self.shader3d
                    .assert_uniform_by_name("uniform_modelview_matrix")?
                    .location,
                1,
                gl::FALSE,
                self.camera.matrix().to_cols_array().as_ptr(),
            );

            gl::ActiveTexture(gl::TEXTURE0);
            self.texture.bind();
            gl::Uniform1i(
                self.shader3d
                    .assert_uniform_by_name("uniform_sampler")?
                    .location,
                0,
            );

            self.cube_vertex_array_object.bind();
            gl::DrawElements(
                gl::TRIANGLES,
                self.cube_element_array_buffer.len() as i32,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );

            gl::Disable(gl::DEPTH_TEST);

            if let Some(font_texture) = &self.font_texture {
                self.shader2d.use_program();

                gl::UniformMatrix4fv(
                    self.shader2d
                        .assert_uniform_by_name("uniform_projection_matrix")?
                        .location,
                    1,
                    gl::FALSE,
                    ortho.to_cols_array().as_ptr(),
                );
                gl::UniformMatrix4fv(
                    self.shader2d
                        .assert_uniform_by_name("uniform_modelview_matrix")?
                        .location,
                    1,
                    gl::FALSE,
                    Mat4::IDENTITY.to_cols_array().as_ptr(),
                );

                gl::ActiveTexture(gl::TEXTURE0);
                font_texture.bind();
                gl::Uniform1i(
                    self.shader2d
                        .assert_uniform_by_name("uniform_sampler")?
                        .location,
                    0,
                );

                self.font_vertex_array_object.bind();
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.font_element_array_buffer.len() as i32,
                    gl::UNSIGNED_SHORT,
                    std::ptr::null(),
                );
            }
        }

        Ok(())
    }

    fn update(
        &mut self,
        AppState { keyboard, .. }: &AppState,
        elapsed_time: Duration,
    ) -> Result<()> {
        // TODO re-use an existing texture instead of re-allocating every time
        let font_texture = self
            .font
            .render_text("Hello, World!\nTODO put an FPS counter here")?;
        let width = font_texture.width();
        let height = font_texture.height();
        self.font_texture = Some(font_texture);
        let color = vec4(1.0, 1.0, 1.0, 1.0);
        // TODO need to have a mesh style system that can automatically recreate the VAO once this succeeds
        self.font_array_buffer.set_data(
            0,
            &[
                Vertex2DColorTextureCoordinate::new(vec2(0.0, 0.0), vec2(0.0, 0.0), color),
                Vertex2DColorTextureCoordinate::new(vec2(width as f32, 0.0), vec2(1.0, 0.0), color),
                Vertex2DColorTextureCoordinate::new(
                    vec2(width as f32, height as f32),
                    vec2(1.0, 1.0),
                    color,
                ),
                Vertex2DColorTextureCoordinate::new(
                    vec2(0.0, height as f32),
                    vec2(0.0, 1.0),
                    color,
                ),
            ],
        )?;
        // TODO make it easy to recreate the VAO
        self.font_vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
            &self.shader2d,
            &self.font_array_buffer,
            &self.font_element_array_buffer,
            &[
                VertexAttributeDefinition {
                    name: "in_position".to_owned(),
                    offset: bytemuck::offset_of!(Vertex2DColorTextureCoordinate, position),
                },
                VertexAttributeDefinition {
                    name: "in_texture_coordinate".to_owned(),
                    offset: bytemuck::offset_of!(
                        Vertex2DColorTextureCoordinate,
                        texture_coordinate
                    ),
                },
                VertexAttributeDefinition {
                    name: "in_color".to_owned(),
                    offset: bytemuck::offset_of!(Vertex2DColorTextureCoordinate, color),
                },
            ],
        )?;

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
