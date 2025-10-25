use std::{
    ffi::{CString, c_void},
    marker::PhantomData,
    thread,
    time::{Duration, SystemTime},
};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::{Result, eyre};
use sdl3::keyboard::Keycode;
use tracing::*;

#[derive(Debug, Clone, Copy)]
enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    pub fn gl_type(self) -> u32 {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

struct Shader {
    instance: u32,
}

impl Shader {
    pub fn new(typ: ShaderType, source: &str) -> Result<Self> {
        info!("compile shader type={:?}, source=\n{}", typ, source);
        unsafe {
            let result = Self {
                instance: gl::CreateShader(typ.gl_type()),
            };

            gl::ShaderSource(
                result.instance,
                1,
                &CString::new(source)?.as_ptr(),
                std::ptr::null(),
            );

            gl::CompileShader(result.instance);

            let mut status = 0;
            gl::GetShaderiv(result.instance, gl::COMPILE_STATUS, &mut status);

            if status == 0 {
                let mut length = 0;
                gl::GetShaderiv(result.instance, gl::INFO_LOG_LENGTH, &mut length);
                let mut c_str = vec![0; length as usize];
                c_str.set_len(length as usize);
                let mut real_length = 0;
                gl::GetShaderInfoLog(
                    result.instance,
                    length,
                    &mut real_length,
                    c_str.as_mut_ptr() as *mut i8,
                );
                Err(eyre!(
                    "shader compile error: {}",
                    CString::from_vec_unchecked(c_str).to_str()?
                ))?;
            }

            Ok(result)
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.instance) };
    }
}

struct ShaderProgram {
    instance: u32,
}

impl ShaderProgram {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Result<Self> {
        let vertex_shader = Shader::new(ShaderType::Vertex, vertex_shader_source)?;
        let fragment_shader = Shader::new(ShaderType::Fragment, fragment_shader_source)?;

        unsafe {
            let result = Self {
                instance: gl::CreateProgram(),
            };
            gl::AttachShader(result.instance, vertex_shader.instance);
            gl::AttachShader(result.instance, fragment_shader.instance);
            gl::LinkProgram(result.instance);

            let mut status = 0;
            gl::GetProgramiv(result.instance, gl::LINK_STATUS, &mut status);

            if status == 0 {
                let mut length = 0;
                gl::GetProgramiv(result.instance, gl::INFO_LOG_LENGTH, &mut length);
                let mut c_str = vec![0; length as usize];
                c_str.set_len(length as usize);
                let mut real_length = 0;
                gl::GetProgramInfoLog(
                    result.instance,
                    length,
                    &mut real_length,
                    c_str.as_mut_ptr() as *mut i8,
                );
                Err(eyre!(
                    "shader program link error: {}",
                    CString::from_vec_unchecked(c_str).to_str()?
                ))?;
            }

            Ok(result)
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.instance);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BufferTarget {
    Array,
    ElementArray,
}

impl BufferTarget {
    pub fn gl_type(self) -> u32 {
        match self {
            BufferTarget::Array => gl::ARRAY_BUFFER,
            BufferTarget::ElementArray => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BufferUsage {
    StaticDraw,
    // TODO rest of the usage enums
}

impl BufferUsage {
    pub fn gl_type(self) -> u32 {
        match self {
            BufferUsage::StaticDraw => gl::STATIC_DRAW,
        }
    }
}

struct Buffer<T> {
    target: BufferTarget,
    instance: u32,
    _phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(target: BufferTarget, usage: BufferUsage, data: &[T]) -> Result<Self>
    where
        T: Pod,
    {
        unsafe {
            let mut instance = 0;
            gl::GenBuffers(1, &mut instance);

            gl::BindBuffer(target.gl_type(), instance);

            let bytes: &[u8] = bytemuck::cast_slice(data);
            gl::BufferData(
                target.gl_type(),
                bytes.len() as isize,
                data.as_ptr() as *mut c_void,
                usage.gl_type(),
            );

            Ok(Self {
                target,
                instance,
                _phantom: Default::default(),
            })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.target.gl_type(), self.instance);
        }
    }
}

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
