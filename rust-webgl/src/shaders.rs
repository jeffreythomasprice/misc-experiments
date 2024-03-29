use std::{collections::HashMap, rc::Rc};

use web_sys::{
    WebGl2RenderingContext, WebGlActiveInfo, WebGlProgram, WebGlShader, WebGlUniformLocation,
};

use crate::errors::JsInteropError;

struct Shader {
    gl: Rc<WebGl2RenderingContext>,
    instance: WebGlShader,
}

impl Shader {
    pub fn new(
        gl: Rc<WebGl2RenderingContext>,
        typ: u32,
        source: &str,
    ) -> Result<Self, JsInteropError> {
        let result = gl
            .create_shader(typ)
            .ok_or(JsInteropError::NotFound(format!(
                "failed to create shader of type {typ}"
            )))?;
        gl.shader_source(&result, source);
        gl.compile_shader(&result);
        let compile_status = gl
            .get_shader_parameter(&result, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .ok_or(JsInteropError::CastError("expected bool".to_owned()))?;
        if compile_status {
            Ok(Self {
                gl: gl.clone(),
                instance: result,
            })
        } else {
            let log = gl.get_shader_info_log(&result);
            gl.delete_shader(Some(&result));
            Err(JsInteropError::WebGl(format!(
                "shader compile error, type: {typ}, log:\n{log:?}"
            )))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.gl.delete_shader(Some(&self.instance));
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
    gl: Rc<WebGl2RenderingContext>,
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
        gl: Rc<WebGl2RenderingContext>,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self, JsInteropError> {
        let vertex_shader = Shader::new(
            gl.clone(),
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_source,
        )?;
        let fragment_shader = Shader::new(
            gl.clone(),
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_source,
        )?;
        let result = gl.create_program().ok_or_else(|| {
            JsInteropError::NotFound("failed to create shader program".to_owned())
        })?;
        gl.attach_shader(&result, &vertex_shader.instance);
        gl.attach_shader(&result, &fragment_shader.instance);
        gl.link_program(&result);
        let link_status = gl
            .get_program_parameter(&result, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .ok_or(JsInteropError::CastError("expected bool".to_owned()))?;
        if link_status {
            let active_attribs = gl
                .get_program_parameter(&result, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
                .as_f64()
                .ok_or(JsInteropError::CastError("expected a number".to_owned()))?
                as u32;
            let mut attributes = HashMap::new();
            for index in 0..active_attribs {
                let info = Info::new(
                    index,
                    gl.get_active_attrib(&result, index)
                        .ok_or(JsInteropError::NotFound(format!(
                            "failed to find attribute on shader with index {index}"
                        )))?,
                );
                attributes.insert(info.name.clone(), info);
            }

            let active_uniforms = gl
                .get_program_parameter(&result, WebGl2RenderingContext::ACTIVE_UNIFORMS)
                .as_f64()
                .ok_or(JsInteropError::CastError("expected a number".to_owned()))?
                as u32;
            let mut uniforms = HashMap::new();
            for index in 0..active_uniforms {
                let info =
                    gl.get_active_uniform(&result, index)
                        .ok_or(JsInteropError::NotFound(format!(
                            "failed to find uniform on shader with index {index}"
                        )))?;
                let location = gl.get_uniform_location(&result, &info.name()).ok_or(
                    JsInteropError::NotFound(format!(
                        "failed to find uniform location on shader with index {index}, name {}",
                        info.name()
                    )),
                )?;
                let info = UniformInfo::new(index, info, location);
                uniforms.insert(info.info.name.clone(), info);
            }

            Ok(Self {
                gl: gl.clone(),
                vertex_shader,
                fragment_shader,
                instance: result,
                attributes,
                uniforms,
            })
        } else {
            let log = gl.get_program_info_log(&result);
            gl.delete_program(Some(&result));
            Err(JsInteropError::WebGl(format!(
                "shader link error, log:\n{log:?}"
            )))
        }
    }

    pub fn use_program(&self) {
        self.gl.use_program(Some(&self.instance));
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
        self.gl.delete_program(Some(&self.instance));
    }
}
