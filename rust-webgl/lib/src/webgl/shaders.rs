use std::{collections::HashMap, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::{errors::Result, glmath::matrix4::Matrix4};

pub struct ShaderAttributeInfo {
    pub name: String,
    pub size: i32,
    pub type_: u32,
    pub location: i32,
}

pub struct ShaderUniformInfo {
    context: Rc<WebGl2RenderingContext>,
    pub name: String,
    pub size: i32,
    pub type_: u32,
    pub location: WebGlUniformLocation,
}

pub struct ShaderProgram {
    context: Rc<WebGl2RenderingContext>,
    program: WebGlProgram,
    attributes: HashMap<String, ShaderAttributeInfo>,
    uniforms: HashMap<String, ShaderUniformInfo>,
}

impl ShaderProgram {
    pub fn new(
        context: Rc<WebGl2RenderingContext>,
        vertex_shdaer_source: &str,
        fragment_shader_source: &str,
    ) -> Result<Self> {
        fn create_shader(
            context: &WebGl2RenderingContext,
            type_: u32,
            type_str: &str,
            source: &str,
        ) -> Result<WebGlShader> {
            let result = context
                .create_shader(type_)
                .ok_or_else(|| format!("failed to create shader of type {type_str}"))?;

            context.shader_source(&result, source);
            context.compile_shader(&result);

            if !context
                .get_shader_parameter(&result, WebGl2RenderingContext::COMPILE_STATUS)
                .is_truthy()
            {
                let log = context.get_shader_info_log(&result);
                context.delete_shader(Some(&result));
                Err(format!(
                    "error compiling shader of type {type_str}: {log:?}"
                ))?;
            }

            Ok(result)
        }

        let vertex_shader = create_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            "VERTEX",
            vertex_shdaer_source,
        )?;
        let fragment_shader = match create_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            "FRAGMENT",
            fragment_shader_source,
        ) {
            Ok(result) => result,
            Err(e) => {
                context.delete_shader(Some(&vertex_shader));
                Err(e)?
            }
        };

        let program = context
            .create_program()
            .ok_or("failed to create shader program".to_string())?;
        context.attach_shader(&program, &vertex_shader);
        context.attach_shader(&program, &fragment_shader);
        context.link_program(&program);
        context.delete_shader(Some(&vertex_shader));
        context.delete_shader(Some(&fragment_shader));

        if !context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .is_truthy()
        {
            let log = context.get_program_info_log(&program);
            context.delete_program(Some(&program));
            Err(format!("error linking shader program: {log:?}"))?;
        }

        let (attributes, uniforms) = Self::get_attributes_and_uniforms(context.clone(), &program)?;

        Ok(Self {
            context,
            program,
            attributes,
            uniforms,
        })
    }

    pub fn get_attribute(&self, name: &str) -> Result<&ShaderAttributeInfo> {
        Ok(self
            .attributes
            .get(name)
            .ok_or_else(|| format!("no such attribute: {name}"))?)
    }

    pub fn get_uniform(&self, name: &str) -> Result<&ShaderUniformInfo> {
        Ok(self
            .uniforms
            .get(name)
            .ok_or_else(|| format!("no such uniform: {name}"))?)
    }

    pub fn use_program(&self) {
        self.context.use_program(Some(&self.program));
    }

    fn get_attributes_and_uniforms(
        context: Rc<WebGl2RenderingContext>,
        program: &WebGlProgram,
    ) -> Result<(
        HashMap<String, ShaderAttributeInfo>,
        HashMap<String, ShaderUniformInfo>,
    )> {
        let active_attributes = context
            .get_program_parameter(program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .ok_or("expected number of active attributes, got non-number")?
            as u32;
        let mut attributes = HashMap::new();
        for i in 0..active_attributes {
            let info = context.get_active_attrib(program, i).ok_or_else(|| format!("expected attribute at index {i} because we think there are {active_attributes} attributes"))?;
            let location = context.get_attrib_location(program, &info.name());
            attributes.insert(
                info.name(),
                ShaderAttributeInfo {
                    name: info.name(),
                    size: info.size(),
                    type_: info.type_(),
                    location,
                },
            );
        }

        let active_uniforms = context
            .get_program_parameter(program, WebGl2RenderingContext::ACTIVE_UNIFORMS)
            .as_f64()
            .ok_or("expected number of active uniforms, got non-number")?
            as u32;
        let mut uniforms = HashMap::new();
        for i in 0..active_uniforms {
            let info = context.get_active_uniform(program,i).ok_or_else(|| format!("expected uniform at index {i} because we think there are {active_uniforms} uniforms"))?;
            let location = context
                .get_uniform_location(program, &info.name())
                .ok_or_else(|| format!("expected uniform {} but no such uniform", info.name()))?;
            uniforms.insert(
                info.name(),
                ShaderUniformInfo {
                    context: context.clone(),
                    name: info.name(),
                    size: info.size(),
                    type_: info.type_(),
                    location,
                },
            );
        }

        Ok((attributes, uniforms))
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.program));
    }
}

impl ShaderUniformInfo {
    pub fn set1i(&self, x: i32) {
        self.context.uniform1i(Some(&self.location), x);
    }

    pub fn set_matrixf(&self, data: &Matrix4<f32>) {
        self.context
            .uniform_matrix4fv_with_f32_array(Some(&self.location), false, data.flatten());
    }

    // TODO various other uniform helpers
}
