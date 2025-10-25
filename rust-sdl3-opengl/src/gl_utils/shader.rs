use std::ffi::CString;

use color_eyre::eyre::{Result, eyre};
use tracing::*;

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
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

pub struct Shader {
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

pub struct ShaderProgram {
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
