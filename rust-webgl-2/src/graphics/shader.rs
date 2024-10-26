use std::{collections::HashMap, sync::Arc};
use web_sys::{
    WebGl2RenderingContext, WebGlActiveInfo, WebGlProgram, WebGlShader, WebGlUniformLocation,
};

use crate::error::Error;

#[derive(Debug)]
enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    pub fn gl_type(&self) -> u32 {
        match self {
            ShaderType::Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
        }
    }
}

struct Shader {
    context: Arc<WebGl2RenderingContext>,
    instance: WebGlShader,
}

impl Shader {
    pub fn new(
        context: Arc<WebGl2RenderingContext>,
        typ: ShaderType,
        source: &str,
    ) -> Result<Self, Error> {
        let result = context
            .create_shader(typ.gl_type())
            .ok_or("failed to create shader of type {typ:?}")?;
        context.shader_source(&result, source);
        context.compile_shader(&result);
        let compile_status = context
            .get_shader_parameter(&result, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .ok_or("expected bool")?;
        if compile_status {
            Ok(Self {
                context: context.clone(),
                instance: result,
            })
        } else {
            let log = context.get_shader_info_log(&result);
            context.delete_shader(Some(&result));
            Err(format!(
                "shader compile error, type: {typ:?}, log:\n{log:?}"
            ))?
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.context.delete_shader(Some(&self.instance));
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub index: u32,
    pub size: i32,
    pub typ: u32,
    pub name: String,
}

impl Info {
    pub fn new(index: u32, info: WebGlActiveInfo) -> Self {
        Self {
            index,
            size: info.size(),
            typ: info.type_(),
            name: info.name(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub context: Arc<WebGl2RenderingContext>,
    pub info: Info,
}

impl Attribute {
    pub fn new(context: Arc<WebGl2RenderingContext>, index: u32, info: WebGlActiveInfo) -> Self {
        Self {
            context,
            info: Info::new(index, info),
        }
    }

    pub fn enable(&self) {
        self.context.enable_vertex_attrib_array(self.info.index);
    }

    pub fn disable(&self) {
        self.context.disable_vertex_attrib_array(self.info.index);
    }
}

#[derive(Debug, Clone)]
pub enum AttributePointerType {
    // TODO the rest of the attribute pointer types
    Float,
}

#[derive(Debug, Clone)]
pub struct AttributePointer {
    attribute: Attribute,
    size: i32,
    typ: AttributePointerType,
    gl_type: u32,
    normalized: bool,
    stride: i32,
    offset: i32,
}

impl AttributePointer {
    pub fn new<T>(
        attribute: Attribute,
        size: i32,
        typ: AttributePointerType,
        normalized: bool,
        offset: i32,
    ) -> Self {
        let gl_type = match typ {
            AttributePointerType::Float => WebGl2RenderingContext::FLOAT,
        };
        Self {
            attribute,
            size,
            typ,
            gl_type,
            normalized,
            stride: size_of::<T>() as i32,
            offset,
        }
    }

    pub fn enable(&self) {
        self.attribute.enable();
        self.attribute.context.vertex_attrib_pointer_with_i32(
            self.attribute.info.index,
            self.size,
            self.gl_type,
            self.normalized,
            self.stride,
            self.offset,
        );
    }

    pub fn disable(&self) {
        self.attribute.disable();
    }
}

#[derive(Debug, Clone)]
pub struct Uniform {
    pub info: Info,
    pub location: WebGlUniformLocation,
}

impl Uniform {
    pub fn new(index: u32, info: WebGlActiveInfo, location: WebGlUniformLocation) -> Self {
        Self {
            info: Info::new(index, info),
            location,
        }
    }
}

pub struct ShaderProgram {
    context: Arc<WebGl2RenderingContext>,
    #[allow(dead_code)]
    vertex_shader: Shader,
    #[allow(dead_code)]
    fragment_shader: Shader,
    instance: WebGlProgram,
    attributes: HashMap<String, Attribute>,
    uniforms: HashMap<String, Uniform>,
}

impl ShaderProgram {
    pub fn new(
        context: Arc<WebGl2RenderingContext>,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self, Error> {
        let vertex_shader = Shader::new(context.clone(), ShaderType::Vertex, vertex_source)?;
        let fragment_shader = Shader::new(context.clone(), ShaderType::Fragment, fragment_source)?;
        let result = context
            .create_program()
            .ok_or("failed to create shader program")?;
        context.attach_shader(&result, &vertex_shader.instance);
        context.attach_shader(&result, &fragment_shader.instance);
        context.link_program(&result);
        let link_status = context
            .get_program_parameter(&result, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .ok_or("expected bool")?;
        if link_status {
            let active_attribs = context
                .get_program_parameter(&result, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
                .as_f64()
                .ok_or("expected a number")? as u32;
            let mut attributes = HashMap::new();
            for index in 0..active_attribs {
                let attr = Attribute::new(
                    context.clone(),
                    index,
                    context.get_active_attrib(&result, index).ok_or(format!(
                        "failed to find attribute on shader with index {index}"
                    ))?,
                );
                attributes.insert(attr.info.name.clone(), attr);
            }

            let active_uniforms = context
                .get_program_parameter(&result, WebGl2RenderingContext::ACTIVE_UNIFORMS)
                .as_f64()
                .ok_or("expected a number")? as u32;
            let mut uniforms = HashMap::new();
            for index in 0..active_uniforms {
                let info = context.get_active_uniform(&result, index).ok_or(format!(
                    "failed to find uniform on shader with index {index}"
                ))?;
                let location =
                    context
                        .get_uniform_location(&result, &info.name())
                        .ok_or(format!(
                            "failed to find uniform location on shader with index {index}, name {}",
                            info.name()
                        ))?;
                let uniform = Uniform::new(index, info, location);
                uniforms.insert(uniform.info.name.clone(), uniform);
            }

            Ok(Self {
                context: context.clone(),
                vertex_shader,
                fragment_shader,
                instance: result,
                attributes,
                uniforms,
            })
        } else {
            let log = context.get_program_info_log(&result);
            context.delete_program(Some(&result));
            Err(format!("shader link error, log:\n{log:?}"))?
        }
    }

    pub fn use_program(&self) {
        self.context.use_program(Some(&self.instance));
    }

    pub fn use_none(&self) {
        self.context.use_program(None);
    }

    pub fn get_attribute_by_name(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }

    pub fn get_uniform_by_name(&self, name: &str) -> Option<&Uniform> {
        self.uniforms.get(name)
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.instance));
    }
}
