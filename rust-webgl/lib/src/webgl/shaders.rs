use std::{collections::HashMap, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::errors::Result;

pub struct AttributeInfo {
    pub name: String,
    pub size: i32,
    pub type_: u32,
    pub location: i32,
}

pub struct ShaderProgram {
    context: Rc<WebGl2RenderingContext>,
    program: WebGlProgram,
    attributes: HashMap<String, AttributeInfo>,
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

        let active_attributes = context
            .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .ok_or("expected number of active attributes, got non-number")?
            as u32;
        let mut attributes = HashMap::new();
        for i in 0..active_attributes {
            let attribute = context.get_active_attrib(&program, i).ok_or_else(|| format!("expected attribute at index {i} because we think there are {active_attributes} attributes"))?;
            let location = context.get_attrib_location(&program, &attribute.name());
            attributes.insert(
                attribute.name(),
                AttributeInfo {
                    name: attribute.name(),
                    size: attribute.size(),
                    type_: attribute.type_(),
                    location,
                },
            );
        }

        Ok(Self {
            context,
            program,
            attributes,
        })
    }

    pub fn get_attribute(&self, name: &str) -> Result<&AttributeInfo> {
        Ok(self
            .attributes
            .get(name)
            .ok_or_else(|| format!("no such attribute: {name}"))?)
    }

    pub fn use_program(&self) {
        self.context.use_program(Some(&self.program));
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.program));
    }
}
