use base64::prelude::BASE64_STANDARD;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use lib::graphics::{Rectangle, Renderer, Size};
use lib::Result;
use web_sys::wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, ImageData, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d, SvgImageElement,
};

pub struct CanavsRenderingContext2dSVG {
    image: HtmlImageElement,
    default_size: Size,
}

pub struct CanvasRenderingContext2dRenderer {
    context: CanvasRenderingContext2d,
}

impl CanvasRenderingContext2dRenderer {
    pub fn new_from_canvas(canvas: &HtmlCanvasElement) -> Result<Self> {
        let result = canvas
            .get_context("2d")
            .map_err(|e| format!("{e:?}"))?
            .ok_or("failed to make 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "created a context element, but it wasn't the expected type")?;
        Ok(Self { context: result })
    }
}

impl Renderer for CanvasRenderingContext2dRenderer {
    type SVG = CanavsRenderingContext2dSVG;

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

    fn new_svg(&self, source: &str) -> Result<Self::SVG> {
        let image = HtmlImageElement::new()
            .map_err(|e| format!("failed to make html image element: {e:?}"))?;
        image.set_src(&format!(
            "data:image/svg+xml;base64,{}",
            BASE64_STANDARD.encode(source)
        ));

        let tree = usvg::Tree::from_str(source, &usvg::Options::default())
            .map_err(|e| format!("failed to parse svg: {e:?}"))?;
        let size = tree.size();

        Ok(Self::SVG {
            image,
            default_size: Size::new(size.width() as f64, size.height() as f64)?,
        })
    }

    fn draw_svg(&mut self, source: &Self::SVG, destination: &Rectangle) -> Result<()> {
        // find the largest rectangle that fits in the given destination bounds with the same aspect ratio as this svg
        let scale = (destination.size().width() / source.default_size.width())
            .min(destination.size().height() / source.default_size.height());
        let size = Size::new(
            source.default_size.width() * scale,
            source.default_size.height() * scale,
        )?;
        let bounds = Rectangle::from_centered_size(destination, size);

        // actually draw
        self.context
            .draw_image_with_html_image_element_and_dw_and_dh(
                &source.image,
                bounds.origin().x,
                bounds.origin().y,
                bounds.size().width(),
                bounds.size().height(),
            )
            .map_err(|e| format!("failed to draw svg image: {e:?}"))?;
        Ok(())
    }
}
