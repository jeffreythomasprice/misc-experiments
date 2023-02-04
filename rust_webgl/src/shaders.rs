use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use crate::app_states::*;

pub struct Shader {
	gl: Rc<WebGl2RenderingContext>,
	program: WebGlProgram,
}

impl Shader {
	pub fn new(
		gl: Rc<WebGl2RenderingContext>,
		vertex_source: &str,
		fragment_source: &str,
	) -> Result<Shader, AppError> {
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
			Err(log.into())
		} else {
			Ok(Shader {
				gl: gl.clone(),
				program,
			})
		}
	}

	pub fn bind(&self) {
		self.gl.use_program(Some(&self.program));
	}

	pub fn bind_none(&self) {
		self.gl.use_program(None);
	}

	pub fn get_attrib_location(&self, name: &str) -> Option<u32> {
		let result = self.gl.get_attrib_location(&self.program, name);
		if result >= 0 {
			Some(result as u32)
		} else {
			None
		}
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
