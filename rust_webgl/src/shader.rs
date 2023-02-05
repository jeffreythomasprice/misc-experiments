use std::{collections::HashMap, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::app_states::*;

#[derive(Debug, Clone)]
pub struct AttributeInfo {
	pub location: u32,
	pub name: String,
	pub size: i32,
	pub gl_type: u32,
}

// TODO uniform attrib too

pub struct Shader {
	gl: Rc<WebGl2RenderingContext>,
	program: WebGlProgram,
	attributes_by_location: HashMap<i32, AttributeInfo>,
	attributes_by_name: HashMap<String, AttributeInfo>,
}

impl Shader {
	pub fn new(
		gl: Rc<WebGl2RenderingContext>,
		vertex_source: &str,
		fragment_source: &str,
	) -> Result<Self, AppError> {
		let vertex_shader =
			compile_shader(&gl, vertex_source, WebGl2RenderingContext::VERTEX_SHADER)?;

		let fragment_shader = match compile_shader(
			&gl,
			fragment_source,
			WebGl2RenderingContext::FRAGMENT_SHADER,
		) {
			Ok(result) => Ok(result),
			Err(e) => {
				gl.delete_shader(Some(&vertex_shader));
				Err(e)
			}
		}?;

		let program = match gl.create_program().ok_or("failed to create shader program") {
			Ok(result) => Ok(result),
			Err(e) => {
				gl.delete_shader(Some(&vertex_shader));
				gl.delete_shader(Some(&fragment_shader));
				Err(e)
			}
		}?;

		gl.attach_shader(&program, &vertex_shader);
		gl.attach_shader(&program, &fragment_shader);
		gl.link_program(&program);
		gl.delete_shader(Some(&vertex_shader));
		gl.delete_shader(Some(&fragment_shader));
		if !gl
			.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
			.as_bool()
			.unwrap()
		{
			let log = gl.get_program_info_log(&program).unwrap();
			gl.delete_program(Some(&program));
			Err(log)?;
		}

		let mut attributes_by_location = HashMap::new();
		let mut attributes_by_name = HashMap::new();
		let active_attributes = gl
			.get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
			.as_f64()
			.unwrap() as usize;
		for i in 0..active_attributes {
			let gl_info = gl.get_active_attrib(&program, i as u32).ok_or(format!(
				"expected attribute at index {} given that there are {} attributes",
				i, active_attributes
			))?;
			let location = gl.get_attrib_location(&program, &gl_info.name());
			if location < 0 {
				Err(format!(
					"failed to get attribute location for {}",
					gl_info.name()
				))?;
			}
			let info = AttributeInfo {
				location: location as u32,
				name: gl_info.name(),
				size: gl_info.size(),
				gl_type: gl_info.type_(),
			};
			let name = &info.name;
			attributes_by_location.insert(location, info.clone());
			attributes_by_name.insert(name.clone(), info);
		}

		Ok(Self {
			gl: gl.clone(),
			program,
			attributes_by_location,
			attributes_by_name,
		})
	}

	pub fn bind(&self) {
		self.gl.use_program(Some(&self.program));
	}

	pub fn bind_none(&self) {
		self.gl.use_program(None);
	}

	pub fn get_attributes_by_location(&self) -> &HashMap<i32, AttributeInfo> {
		&self.attributes_by_location
	}

	pub fn get_attributes_by_name(&self) -> &HashMap<String, AttributeInfo> {
		&self.attributes_by_name
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.gl.delete_program(Some(&self.program));
	}
}

fn compile_shader(
	gl: &WebGl2RenderingContext,
	source: &str,
	shader_type: u32,
) -> Result<WebGlShader, AppError> {
	let result = gl
		.create_shader(shader_type)
		.ok_or(format!("failed to create shader of type: {shader_type}"))?;

	gl.shader_source(&result, source);
	gl.compile_shader(&result);

	let status = gl
		.get_shader_parameter(&result, WebGl2RenderingContext::COMPILE_STATUS)
		.as_bool()
		.unwrap();
	if !status {
		let log = gl.get_shader_info_log(&result).unwrap();
		gl.delete_shader(Some(&result));
		let shader_type_str = match shader_type {
			WebGl2RenderingContext::VERTEX_SHADER => "VERTEX".into(),
			WebGl2RenderingContext::FRAGMENT_SHADER => "FRAGMENT".into(),
			_ => format!("{shader_type}"),
		};
		Err(format!("error compiling shader of type {shader_type_str}:\n{log}").into())
	} else {
		Ok(result)
	}
}
