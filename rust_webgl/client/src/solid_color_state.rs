use std::{rc::Rc, time::Duration};

use vek::Extent2;
use vek::Rgba;
use web_sys::WebGl2RenderingContext;

use lib::*;

pub struct SolidColorState {
	color: Rgba<f32>,
	gl: Option<Rc<WebGl2RenderingContext>>,
}

impl SolidColorState {
	pub fn new(color: Rgba<f32>) -> Self {
		Self { color, gl: None }
	}
}

impl AppState for SolidColorState {
	fn activate(&mut self, gl: Rc<WebGl2RenderingContext>) -> AppResult<()> {
		self.gl = Some(gl.clone());
		Ok(())
	}

	fn deactivate(&mut self) -> AppResult<()> {
		Ok(())
	}

	fn resize(&mut self, size: Extent2<i32>) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();

		gl.viewport(0, 0, size.w, size.h);

		Ok(())
	}

	fn render(&mut self) -> AppResult<()> {
		let gl = self.gl.clone().unwrap();

		gl.clear_color(self.color.r, self.color.g, self.color.b, self.color.a);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

		Ok(())
	}

	fn update(&mut self, _time: Duration) -> AppResult<Option<AppStateHandle>> {
		Ok(None)
	}
}
