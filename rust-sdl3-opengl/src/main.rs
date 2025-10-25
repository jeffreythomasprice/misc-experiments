mod gl_utils;

use std::{thread, time::SystemTime};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::{Result, eyre};
use sdl3::keyboard::Keycode;

use crate::gl_utils::{
    buffer::{Buffer, BufferTarget, BufferUsage},
    shader::ShaderProgram,
};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    pub x: f32,
    pub y: f32,
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
            Vertex { x: -0.5, y: -0.5 },
            Vertex { x: 0.5, y: -0.5 },
            Vertex { x: 0.0, y: 0.5 },
        ],
    )?;

    // TODO structify vertex array objects
    let mut vertex_array_object = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vertex_array_object);
        gl::BindVertexArray(vertex_array_object);
        // TODO helper for figuring out the vertex attributes from the shader and vertex
        gl::EnableVertexAttribArray(0);
        array_buffer.bind();
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
    }

    let mut last_tick = None;
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
            gl::BindVertexArray(vertex_array_object);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        window.gl_swap_window();

        let now = SystemTime::now();
        if let Some(previous) = last_tick {
            // TODO sensible FPS calcualtor
            // const desired_fps: f64 = 60.0;
            // const milliseconds_per_frame: f64 = 1000.0 / desired_fps;
            // let elapsed_time = now - previous;
            // if elapsed_time < milliseconds_per_frame {

            //     // thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            // }
            thread::yield_now();
        }
        last_tick = Some(now);
    }

    Ok(())
}
