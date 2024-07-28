use super::{Point, RGBColor, Rectangle};

pub trait Renderer {
    fn set_stroke_color(&mut self, c: &RGBColor);
    fn set_line_width(&mut self, x: f64);
    fn set_fill_color(&mut self, c: &RGBColor);
    fn begin_path(&mut self);
    fn move_to(&mut self, p: &Point);
    fn line_to(&mut self, p: &Point);
    fn quadratic_curve_to(&mut self, c: &Point, p: &Point);
    fn bezier_curve_to(&mut self, c1: &Point, c2: &Point, p: &Point);
    fn close_path(&mut self);
    fn stroke(&mut self);
    fn fill(&mut self);
}

pub fn add_rectangle_path<R>(renderer: &mut R, bounds: &Rectangle)
where
    R: Renderer,
{
    renderer.begin_path();
    renderer.move_to(bounds.min());
    renderer.line_to(&Point {
        x: bounds.max().x,
        y: bounds.min().y,
    });
    renderer.line_to(&bounds.max());
    renderer.line_to(&Point {
        x: bounds.min().x,
        y: bounds.max().y,
    });
    renderer.close_path();
}

pub fn stroke_line<R>(renderer: &mut R, p1: &Point, p2: &Point, c: &RGBColor, width: f64)
where
    R: Renderer,
{
    renderer.begin_path();
    renderer.move_to(p1);
    renderer.line_to(p2);
    renderer.set_stroke_color(c);
    renderer.set_line_width(width);
    renderer.stroke();
}

pub fn stroke_rectangle<R>(renderer: &mut R, bounds: &Rectangle, c: &RGBColor, width: f64)
where
    R: Renderer,
{
    add_rectangle_path(renderer, bounds);
    renderer.set_stroke_color(c);
    renderer.set_line_width(width);
    renderer.stroke();
}

pub fn fill_rectangle<R>(renderer: &mut R, bounds: &Rectangle, c: &RGBColor)
where
    R: Renderer,
{
    add_rectangle_path(renderer, bounds);
    renderer.set_fill_color(c);
    renderer.fill();
}
