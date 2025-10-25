mod camera;
mod gl_utils;

use core::f32;
use std::{
    collections::HashMap,
    thread,
    time::{Duration, Instant},
};
use tracing::*;

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    sys::{
        mouse::{SDL_HideCursor, SDL_ShowCursor, SDL_WarpMouseInWindow},
        video::{
            SDL_GL_SetSwapInterval, SDL_SetWindowSurfaceVSync, SDL_WINDOW_SURFACE_VSYNC_DISABLED,
        },
    },
};

use crate::{
    camera::Camera,
    gl_utils::{
        buffer::{Buffer, BufferTarget, BufferUsage},
        shader::ShaderProgram,
        texture::Texture,
        vertex_array_object::VertexArrayObject,
    },
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

struct KeyboardState {
    state: HashMap<Keycode, bool>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub fn is_pressed(&self, keycode: Keycode) -> bool {
        *self.state.get(&keycode).unwrap_or(&false)
    }

    pub fn set_pressed(&mut self, keycode: Keycode, pressed: bool) {
        self.state.insert(keycode, pressed);
    }
}

fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    let sdl_context = sdl3::init()?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Experiment", 1024, 768)
        .position_centered()
        .opengl()
        .build()?;

    let gl_context = window.gl_create_context()?;
    window.gl_make_current(&gl_context)?;

    // turn off vsync
    unsafe {
        SDL_SetWindowSurfaceVSync(window.raw(), SDL_WINDOW_SURFACE_VSYNC_DISABLED);
        SDL_GL_SetSwapInterval(0);
    }

    gl::load_with(|s| {
        video_subsystem
            .gl_get_proc_address(s)
            .ok_or(eyre!("failed to get GL proc address: {}", s))
            .unwrap() as *const _
    });

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
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
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

    let mut camera = Camera::new(
        vec3(0.0, 0.0, 6.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    );

    let mut is_mouse_locked = false;
    let mut keyboard_state = KeyboardState::new();

    const DESIRED_FPS: f64 = 60.0;
    const DESIRED_FRAME_DURATION: Duration = Duration::from_nanos(
        ((Duration::from_secs(1).as_nanos() as f64) / DESIRED_FPS).ceil() as u64,
    );
    let mut last_tick: Option<Instant> = None;

    'mainLoop: loop {
        let mut event_pump = sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainLoop,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    keyboard_state.set_pressed(keycode, true);
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    keyboard_state.set_pressed(keycode, false);
                }

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    is_mouse_locked = !is_mouse_locked;
                    unsafe {
                        if is_mouse_locked {
                            SDL_HideCursor();
                        } else {
                            SDL_ShowCursor();
                        }
                    }
                }

                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    if is_mouse_locked {
                        let (width, height) = window.size();
                        let center_x = (width as f32) / 2.0;
                        let center_y = (height as f32) / 2.0;

                        if !(x == center_x && y == center_y) {
                            camera.turn(vec2(xrel, yrel));
                        }

                        unsafe {
                            SDL_WarpMouseInWindow(window.raw(), center_x, center_y);
                        }
                    }
                }

                _ => (),
            };
        }

        unsafe {
            gl::ClearColor(0.25, 0.5, 0.75, 1.0);
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::DEPTH_TEST);

            shader.use_program();

            let (width, height) = window.size();
            let ortho = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
            let perspective = Mat4::perspective_lh(
                f32::consts::FRAC_PI_6,
                (width as f32) / (height as f32),
                0.01,
                1000.0,
            );
            gl::UniformMatrix4fv(
                shader
                    .assert_uniform_by_name("uniform_projection_matrix")?
                    .location,
                1,
                gl::FALSE,
                perspective.to_cols_array().as_ptr(),
            );
            gl::UniformMatrix4fv(
                shader
                    .assert_uniform_by_name("uniform_modelview_matrix")?
                    .location,
                1,
                gl::FALSE,
                camera.matrix().to_cols_array().as_ptr(),
            );

            gl::ActiveTexture(gl::TEXTURE0);
            texture.bind();
            gl::Uniform1i(
                shader.assert_uniform_by_name("uniform_sampler")?.location,
                0,
            );

            vertex_array_object.bind();
            gl::DrawElements(
                gl::TRIANGLES,
                element_array_buffer.len() as i32,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }
        window.gl_swap_window();

        let now = Instant::now();
        if let Some(previous) = last_tick {
            let elapsed_time = now - previous;

            let elapsed_seconds = elapsed_time.as_secs_f32();
            let left =
                keyboard_state.is_pressed(Keycode::Left) || keyboard_state.is_pressed(Keycode::A);
            let right =
                keyboard_state.is_pressed(Keycode::Right) || keyboard_state.is_pressed(Keycode::D);
            let forward =
                keyboard_state.is_pressed(Keycode::Up) || keyboard_state.is_pressed(Keycode::W);
            let backward =
                keyboard_state.is_pressed(Keycode::Down) || keyboard_state.is_pressed(Keycode::S);
            let up = keyboard_state.is_pressed(Keycode::Space);
            let down = keyboard_state.is_pressed(Keycode::LShift)
                || keyboard_state.is_pressed(Keycode::RShift);
            let move_speed = elapsed_seconds * 10.0;
            camera.move_position(
                (if forward { 1.0 } else { 0.0 } - if backward { 1.0 } else { 0.0 }) * move_speed,
                (if right { 1.0 } else { 0.0 } - if left { 1.0 } else { 0.0 }) * move_speed,
                (if up { 1.0 } else { 0.0 } - if down { 1.0 } else { 0.0 }) * move_speed,
            );

            if elapsed_time >= DESIRED_FRAME_DURATION {
                thread::yield_now();
            } else {
                thread::sleep(DESIRED_FRAME_DURATION - elapsed_time);
            }
        }
        last_tick = Some(now);
    }

    Ok(())
}
