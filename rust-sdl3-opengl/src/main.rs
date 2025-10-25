mod gl_utils;

use std::{
    ffi::c_void,
    thread,
    time::{Duration, Instant, SystemTime},
};
use tracing::*;

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, Vec2, Vec4, vec2, vec4};
use sdl3::{
    keyboard::Keycode,
    sys::video::{
        SDL_GL_SetSwapInterval, SDL_SetWindowSurfaceVSync, SDL_WINDOW_SURFACE_VSYNC_DISABLED,
    },
};

use crate::gl_utils::{
    buffer::{Buffer, BufferTarget, BufferUsage},
    shader::ShaderProgram,
    vertex_array_object::VertexArrayObject,
};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
struct Vertex {
    pub position: Vec2,
    _padding: [u8; 8],
    pub color: Vec4,
}

impl Vertex {
    pub fn new(position: Vec2, color: Vec4) -> Self {
        Self {
            position,
            _padding: Default::default(),
            color,
        }
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

    let shader = ShaderProgram::new(
        include_str!("shaders/shader.vert"),
        include_str!("shaders/shader.frag"),
    )?;

    let array_buffer = Buffer::new(
        BufferTarget::Array,
        BufferUsage::StaticDraw,
        &[
            Vertex::new(vec2(50.0, 50.0), vec4(0.5, 0.25, 0.0, 1.0)),
            Vertex::new(vec2(300.0, 50.0), vec4(0.0, 0.5, 0.25, 1.0)),
            Vertex::new(vec2(300.0, 300.0), vec4(0.25, 0.0, 0.5, 1.0)),
            Vertex::new(vec2(50.0, 300.0), vec4(0.25, 0.5, 0.0, 1.0)),
        ],
    )?;

    let element_array_buffer = Buffer::<u16>::new(
        BufferTarget::ElementArray,
        BufferUsage::StaticDraw,
        &[0, 1, 2, 2, 3, 0],
    )?;

    // TODO builder pattern?
    let vertex_array_object = VertexArrayObject::new_array_and_element_array_buffers(
        &shader,
        &array_buffer,
        &element_array_buffer,
        &[
            ("in_position", bytemuck::offset_of!(Vertex, position)),
            ("in_color", bytemuck::offset_of!(Vertex, color)),
        ],
    )?;

    const DESIRED_FPS: f64 = 60.0;
    const DESIRED_FRAME_DURATION: Duration = Duration::from_nanos(
        ((Duration::from_secs(1).as_nanos() as f64) / DESIRED_FPS).ceil() as u64,
    );
    let mut last_tick: Option<Instant> = None;
    'mainLoop: loop {
        let mut event_pump = sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                sdl3::event::Event::Quit { .. }
                | sdl3::event::Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainLoop,
                _ => (),
            };
        }

        unsafe {
            gl::ClearColor(0.25, 0.5, 0.75, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            shader.use_program();

            // TODO matrices
            let (width, height) = window.size();
            gl::UniformMatrix4fv(
                shader
                    .assert_uniform_by_name("uniform_projection_matrix")?
                    .location,
                1,
                gl::FALSE,
                Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
                    .to_cols_array()
                    .as_ptr(),
            );
            gl::UniformMatrix4fv(
                shader
                    .assert_uniform_by_name("uniform_modelview_matrix")?
                    .location,
                1,
                gl::FALSE,
                Mat4::IDENTITY.to_cols_array().as_ptr(),
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
