use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use web_sys::{
    WebGl2RenderingContext, WebGlActiveInfo, WebGlProgram, WebGlShader, WebGlUniformLocation,
};

struct Shader {
    context: Arc<WebGl2RenderingContext>,
    instance: WebGlShader,
}

impl Shader {
    pub fn new(context: Arc<WebGl2RenderingContext>, typ: u32, source: &str) -> Result<Self> {
        let result = context
            .create_shader(typ)
            .ok_or(anyhow!("failed to create shader of type {typ}"))?;
        context.shader_source(&result, source);
        context.compile_shader(&result);
        let compile_status = context
            .get_shader_parameter(&result, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .ok_or(anyhow!("expected bool"))?;
        if compile_status {
            Ok(Self {
                context: context.clone(),
                instance: result,
            })
        } else {
            let log = context.get_shader_info_log(&result);
            context.delete_shader(Some(&result));
            Err(anyhow!("shader compile error, type: {typ}, log:\n{log:?}"))
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
pub struct UniformInfo {
    pub info: Info,
    pub location: WebGlUniformLocation,
}

impl UniformInfo {
    pub fn new(index: u32, info: WebGlActiveInfo, location: WebGlUniformLocation) -> Self {
        Self {
            info: Info::new(index, info),
            location,
        }
    }
}

pub struct ShaderProgram {
    context: Arc<WebGl2RenderingContext>,
    // TODO needs allow dead code?
    #[allow(dead_code)]
    vertex_shader: Shader,
    #[allow(dead_code)]
    fragment_shader: Shader,
    instance: WebGlProgram,
    attributes: HashMap<String, Info>,
    uniforms: HashMap<String, UniformInfo>,
}

impl ShaderProgram {
    pub fn new(
        context: Arc<WebGl2RenderingContext>,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self> {
        let vertex_shader = Shader::new(
            context.clone(),
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_source,
        )?;
        let fragment_shader = Shader::new(
            context.clone(),
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_source,
        )?;
        let result = context
            .create_program()
            .ok_or_else(|| anyhow!("failed to create shader program"))?;
        context.attach_shader(&result, &vertex_shader.instance);
        context.attach_shader(&result, &fragment_shader.instance);
        context.link_program(&result);
        let link_status = context
            .get_program_parameter(&result, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .ok_or(anyhow!("expected bool"))?;
        if link_status {
            let active_attribs = context
                .get_program_parameter(&result, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
                .as_f64()
                .ok_or(anyhow!("expected a number"))? as u32;
            let mut attributes = HashMap::new();
            for index in 0..active_attribs {
                let info = Info::new(
                    index,
                    context.get_active_attrib(&result, index).ok_or(anyhow!(
                        "failed to find attribute on shader with index {index}"
                    ))?,
                );
                attributes.insert(info.name.clone(), info);
            }

            let active_uniforms = context
                .get_program_parameter(&result, WebGl2RenderingContext::ACTIVE_UNIFORMS)
                .as_f64()
                .ok_or(anyhow!("expected a number"))? as u32;
            let mut uniforms = HashMap::new();
            for index in 0..active_uniforms {
                let info = context.get_active_uniform(&result, index).ok_or(anyhow!(
                    "failed to find uniform on shader with index {index}"
                ))?;
                let location =
                    context
                        .get_uniform_location(&result, &info.name())
                        .ok_or(anyhow!(
                            "failed to find uniform location on shader with index {index}, name {}",
                            info.name()
                        ))?;
                let info = UniformInfo::new(index, info, location);
                uniforms.insert(info.info.name.clone(), info);
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
            Err(anyhow!("shader link error, log:\n{log:?}"))
        }
    }

    pub fn use_program(&self) {
        self.context.use_program(Some(&self.instance));
    }

    pub fn use_none(&self) {
        self.context.use_program(None);
    }

    pub fn get_attribute_by_name(&self, name: &str) -> Option<&Info> {
        self.attributes.get(name)
    }

    pub fn get_uniform_by_name(&self, name: &str) -> Option<&UniformInfo> {
        self.uniforms.get(name)
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.instance));
    }
}
