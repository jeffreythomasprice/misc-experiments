use std::ffi::CString;

use color_eyre::eyre::{Result, eyre};
use gl::types::GLsizei;
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

#[derive(Debug, Clone, Copy)]
pub enum ShaderDataType {
    Float,
    FloatVec2,
    FloatVec3,
    FloatVec4,
    FloatMat2,
    FloatMat3,
    FloatMat4,
    /*
    TODO rest of the possible attribute types
    GL_FLOAT_MAT2x3
    GL_FLOAT_MAT2x4
    GL_FLOAT_MAT3x2
    GL_FLOAT_MAT3x4
    GL_FLOAT_MAT4x2
    GL_FLOAT_MAT4x3
    GL_INT
    GL_INT_VEC2
    GL_INT_VEC3
    GL_INT_VEC4
    GL_UNSIGNED_INT
    GL_UNSIGNED_INT_VEC2
    GL_UNSIGNED_INT_VEC3
    GL_UNSIGNED_INT_VEC4
    GL_DOUBLE
    GL_DOUBLE_VEC2
    GL_DOUBLE_VEC3
    GL_DOUBLE_VEC4
    GL_DOUBLE_MAT2
    GL_DOUBLE_MAT3
    GL_DOUBLE_MAT4
    GL_DOUBLE_MAT2x3
    GL_DOUBLE_MAT2x4
    GL_DOUBLE_MAT3x2
    GL_DOUBLE_MAT3x4
    GL_DOUBLE_MAT4x2
    GL_DOUBLE_MAT4x3
        */
}

impl ShaderDataType {
    pub fn gl_type(self) -> u32 {
        match self {
            Self::Float => gl::FLOAT,
            Self::FloatVec2 => gl::FLOAT_VEC2,
            Self::FloatVec3 => gl::FLOAT_VEC3,
            Self::FloatVec4 => gl::FLOAT_VEC4,
            Self::FloatMat2 => gl::FLOAT_MAT2,
            Self::FloatMat3 => gl::FLOAT_MAT3,
            Self::FloatMat4 => gl::FLOAT_MAT4,
        }
    }
}

impl TryFrom<u32> for ShaderDataType {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            gl::FLOAT => Ok(Self::Float),
            gl::FLOAT_VEC2 => Ok(Self::FloatVec2),
            gl::FLOAT_VEC3 => Ok(Self::FloatVec3),
            gl::FLOAT_VEC4 => Ok(Self::FloatVec4),
            gl::FLOAT_MAT2 => Ok(Self::FloatMat2),
            gl::FLOAT_MAT3 => Ok(Self::FloatMat3),
            gl::FLOAT_MAT4 => Ok(Self::FloatMat4),
            _ => Err(eyre!(
                "unhandled opengl enum for shader attribute type: {}",
                value
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderAttribute {
    pub name: String,
    pub size: i32,
    pub typ: ShaderDataType,
    pub location: u32,
}

#[derive(Debug, Clone)]
pub struct ShaderUniform {
    pub name: String,
    pub size: i32,
    pub typ: ShaderDataType,
    pub location: i32,
}

pub struct ShaderProgram {
    instance: u32,
    pub attributes: Vec<ShaderAttribute>,
    pub uniforms: Vec<ShaderUniform>,
}

impl ShaderProgram {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Result<Self> {
        let vertex_shader = Shader::new(ShaderType::Vertex, vertex_shader_source)?;
        let fragment_shader = Shader::new(ShaderType::Fragment, fragment_shader_source)?;

        unsafe {
            let mut result = Self {
                instance: gl::CreateProgram(),
                attributes: Vec::new(),
                uniforms: Vec::new(),
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

            let mut num_attributes = 0;
            gl::GetProgramiv(result.instance, gl::ACTIVE_ATTRIBUTES, &mut num_attributes);
            trace!("num_attributes = {}", num_attributes);
            for i in 0..(num_attributes as u32) {
                let mut name_c_str = vec![0; 256_usize];
                let mut name_len = 0;
                let mut size = 0;
                let mut typ = 0;
                gl::GetActiveAttrib(
                    result.instance,
                    i,
                    name_c_str.len() as GLsizei,
                    &mut name_len,
                    &mut size,
                    &mut typ,
                    name_c_str.as_mut_ptr() as *mut i8,
                );
                let location =
                    gl::GetAttribLocation(result.instance, name_c_str.as_mut_ptr() as *mut i8);
                name_c_str.resize(name_len as usize, 0);
                let name = CString::from_vec_unchecked(name_c_str).into_string()?;
                let typ: ShaderDataType = typ.try_into()?;
                let attribute = ShaderAttribute {
                    name,
                    size,
                    typ,
                    location: location as u32,
                };
                trace!("attribute[{i}]: {attribute:?}");
                result.attributes.push(attribute);
            }

            let mut num_uniforms = 0;
            gl::GetProgramiv(result.instance, gl::ACTIVE_UNIFORMS, &mut num_uniforms);
            trace!("num_uniforms = {}", num_uniforms);
            for i in 0..(num_uniforms as u32) {
                let mut name_c_str = vec![0; 256_usize];
                let mut name_len = 0;
                let mut size = 0;
                let mut typ = 0;
                gl::GetActiveUniform(
                    result.instance,
                    i,
                    name_c_str.len() as GLsizei,
                    &mut name_len,
                    &mut size,
                    &mut typ,
                    name_c_str.as_mut_ptr() as *mut i8,
                );
                let location =
                    gl::GetUniformLocation(result.instance, name_c_str.as_mut_ptr() as *mut i8);
                name_c_str.resize(name_len as usize, 0);
                let name = CString::from_vec_unchecked(name_c_str).into_string()?;
                let typ: ShaderDataType = typ.try_into()?;
                let uniform = ShaderUniform {
                    name,
                    size,
                    typ,
                    location,
                };
                trace!("uniform[{i}]: {uniform:?}");
                result.uniforms.push(uniform);
            }

            Ok(result)
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.instance);
        }
    }

    pub fn get_attribute_by_name(&self, name: &str) -> Option<&ShaderAttribute> {
        self.attributes.iter().find(|x| x.name == name)
    }

    pub fn assert_attribute_by_name(&self, name: &str) -> Result<&ShaderAttribute> {
        self.get_attribute_by_name(name)
            .ok_or(eyre!("no such attribute: {name}"))
    }

    pub fn get_uniform_by_name(&self, name: &str) -> Option<&ShaderUniform> {
        self.uniforms.iter().find(|x| x.name == name)
    }

    pub fn assert_uniform_by_name(&self, name: &str) -> Result<&ShaderUniform> {
        self.get_uniform_by_name(name)
            .ok_or(eyre!("no such uniform: {name}"))
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.instance);
        }
    }
}
