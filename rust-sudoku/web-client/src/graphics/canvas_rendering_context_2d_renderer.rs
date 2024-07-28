use lib::graphics::Renderer;
use lib::Result;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct CanvasRenderingContext2dRenderer {
    context: CanvasRenderingContext2d,
}

impl CanvasRenderingContext2dRenderer {
    pub fn new_from_canvas(canvas: &HtmlCanvasElement) -> Result<Self> {
        let result = canvas
            .get_context("2d")
            .map_err(|e| format!("{e:?}"))?
            .ok_or("failed to make 2d context")?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| "created a context element, but it wasn't the expected type")?;
        Ok(Self { context: result })
    }
}

impl Renderer for CanvasRenderingContext2dRenderer {
    fn set_stroke_color(&mut self, c: &lib::graphics::RGBColor) {
        self.context
            .set_stroke_style(&format!("rgb({},{},{})", c.red, c.green, c.blue).into());
    }

    fn set_line_width(&mut self, x: f64) {
        self.context.set_line_width(x);
    }

    fn set_fill_color(&mut self, c: &lib::graphics::RGBColor) {
        self.context
            .set_fill_style(&format!("rgb({},{},{})", c.red, c.green, c.blue).into());
    }

    fn begin_path(&mut self) {
        self.context.begin_path();
    }

    fn move_to(&mut self, p: &lib::graphics::Point) {
        self.context.move_to(p.x, p.y);
    }

    fn line_to(&mut self, p: &lib::graphics::Point) {
        self.context.line_to(p.x, p.y);
    }

    fn quadratic_curve_to(&mut self, c: &lib::graphics::Point, p: &lib::graphics::Point) {
        self.context.quadratic_curve_to(c.x, c.y, p.x, p.y);
    }

    fn bezier_curve_to(
        &mut self,
        c1: &lib::graphics::Point,
        c2: &lib::graphics::Point,
        p: &lib::graphics::Point,
    ) {
        self.context
            .bezier_curve_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y);
    }

    fn close_path(&mut self) {
        self.context.close_path();
    }

    fn stroke(&mut self) {
        self.context.stroke();
    }

    fn fill(&mut self) {
        self.context.fill();
    }
}
