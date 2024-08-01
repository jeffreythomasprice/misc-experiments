use lib::graphics::{Rectangle, Renderer, Size};
use lib::Result;
use web_sys::wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, ImageData, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};

pub struct CanavsRenderingContext2dSVG {
    tree: usvg::Tree,
}

impl CanavsRenderingContext2dSVG {
    pub fn render_to_offscreen_canvas(&self, desired_size: &Size) -> Result<OffscreenCanvas> {
        let size = self.tree.size();

        let scale = (desired_size.width() / (size.width() as f64))
            .min(desired_size.height() / (size.height() as f64));

        let desired_pixmap_size = tiny_skia::IntSize::from_wh(
            desired_size.width().ceil() as u32,
            desired_size.height().ceil() as u32,
        )
        .ok_or("failed to get desired size out of input bounding rectangle")?;

        let mut pixmap =
            tiny_skia::Pixmap::new(desired_pixmap_size.width(), desired_pixmap_size.height())
                .ok_or("failed to create temporary pixmap")?;
        resvg::render(
            &self.tree,
            tiny_skia::Transform::from_scale(scale as f32, scale as f32),
            &mut pixmap.as_mut(),
        );

        let image_data = ImageData::new_with_u8_clamped_array(
            Clamped(pixmap.data()),
            desired_pixmap_size.width(),
        )
        .map_err(|e| format!("failed to create image data from svg pixmap: {e:?}"))?;

        let offscreen_canvas =
            OffscreenCanvas::new(desired_pixmap_size.width(), desired_pixmap_size.height())
                .map_err(|e| format!("failed to create offscreen canvas for image data: {e:?}"))?;
        let offscreen_context = offscreen_canvas
            .get_context("2d")
            .map_err(|e| format!("failed to get context for offscreen canvas: {e:?}"))?
            .ok_or("no value for offscreen canvas context")?
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .map_err(|_| "created an offscreen context element, but it wasn't the expected type")?;
        offscreen_context
            .put_image_data(&image_data, 0.0, 0.0)
            .map_err(|e| format!("error putting image data to offscreen context: {e:?}"))?;

        Ok(offscreen_canvas)
    }
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
        let tree = usvg::Tree::from_str(source, &usvg::Options::default())
            .map_err(|e| format!("failed to parse svg: {e:?}"))?;
        Ok(Self::SVG { tree })
    }

    fn draw_svg(&mut self, source: &Self::SVG, destination: &Rectangle) -> Result<()> {
        // TODO cache a bunch of different sizes and scale the best one to the destination
        let temp = source.render_to_offscreen_canvas(destination.size())?;
        // TODO put at the right location
        self.context
            .draw_image_with_offscreen_canvas(&temp, 0.0, 0.0)
            .map_err(|e| format!("failed to draw offscreen canvas for svg: {e:?}"))?;
        Ok(())
    }
}
