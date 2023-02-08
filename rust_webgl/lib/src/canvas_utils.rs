use vek::Extent2;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{document, AppError};

pub fn new_canvas_image<F>(size: Extent2<u32>, f: F) -> Result<HtmlCanvasElement, AppError>
where
	F: Fn(&CanvasRenderingContext2d, Extent2<u32>) -> Result<(), AppError>,
{
	let result = document()?
		.create_element("canvas")?
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.or(Err("failed to cast into the right type"))?;
	result.set_width(size.w);
	result.set_height(size.h);

	let context = result
		.get_context("2d")?
		.ok_or("failed to create canvas 2d context")?
		.dyn_into::<web_sys::CanvasRenderingContext2d>()
		.or(Err("failed to cast into the right type"))?;

	f(&context, size)?;

	Ok(result)
}
