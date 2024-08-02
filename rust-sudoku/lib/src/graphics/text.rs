use super::{Point, Rectangle, Renderer, Size};
use crate::Result;

#[derive(Debug, Clone, Copy)]
pub enum HorizontalStringAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalStringAlign {
    Top,
    Center,
    Bottom,
}

struct OutlineBuilder<'a, R>
where
    R: Renderer,
{
    renderer: &'a mut R,
    position: Point,
}

impl<'a, R> rusttype::OutlineBuilder for OutlineBuilder<'a, R>
where
    R: Renderer,
{
    fn move_to(&mut self, x: f32, y: f32) {
        self.renderer.move_to(&Point {
            x: x as f64 + self.position.x,
            y: y as f64 + self.position.y,
        });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.renderer.line_to(&Point {
            x: x as f64 + self.position.x,
            y: y as f64 + self.position.y,
        });
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.renderer.quadratic_curve_to(
            &Point {
                x: x1 as f64 + self.position.x,
                y: y1 as f64 + self.position.y,
            },
            &Point {
                x: x as f64 + self.position.x,
                y: y as f64 + self.position.y,
            },
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.renderer.bezier_curve_to(
            &Point {
                x: x1 as f64 + self.position.x,
                y: y1 as f64 + self.position.y,
            },
            &Point {
                x: x2 as f64 + self.position.x,
                y: y2 as f64 + self.position.y,
            },
            &Point {
                x: x as f64 + self.position.x,
                y: y as f64 + self.position.y,
            },
        );
    }

    fn close(&mut self) {
        self.renderer.close_path();
    }
}

pub fn fill_string<R>(
    renderer: &mut R,
    s: &str,
    destination_bounds: &Rectangle,
    font: &rusttype::Font,
    scale: rusttype::Scale,
    horizontal_align: HorizontalStringAlign,
    vertical_align: VerticalStringAlign,
) -> Result<()>
where
    R: Renderer,
{
    let text_bounds = measure_text(font, scale, s)?;
    let width_diff = destination_bounds.size().width() - text_bounds.size().width();
    let height_diff = destination_bounds.size().height() - text_bounds.size().height();
    let x = destination_bounds.min().x
        + text_bounds.min().x
        + width_diff
            * match horizontal_align {
                HorizontalStringAlign::Left => 0.0,
                HorizontalStringAlign::Center => 0.5,
                HorizontalStringAlign::Right => 1.0,
            };
    let y = destination_bounds.min().y
        + text_bounds.min().y
        + height_diff
            * match vertical_align {
                VerticalStringAlign::Top => 0.0,
                VerticalStringAlign::Center => 0.5,
                VerticalStringAlign::Bottom => 1.0,
            }
        + text_bounds.size().height();
    for glyph in font.layout(
        s,
        scale,
        rusttype::Point {
            x: x as f32,
            y: y as f32,
        },
    ) {
        renderer.begin_path();
        {
            let mut builder = OutlineBuilder {
                renderer,
                position: Point {
                    x: glyph.position().x as f64,
                    y: glyph.position().y as f64,
                },
            };
            glyph.build_outline(&mut builder);
        }
        renderer.close_path();
    }
    renderer.fill();
    Ok(())
}

/// As fit_str_to_size but the biggest font such that none of the strings are larger than size.
pub fn fit_strings_to_size(
    strings: &Vec<String>,
    destination: &Size,
    font: &rusttype::Font,
) -> Result<Option<rusttype::Scale>> {
    let possible_results = strings
        .iter()
        .map(|s| fit_str_to_size(s, destination, font))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(possible_results
        .iter()
        .min_by(|a, b| {
            // assume x and y are the same, because we should be using uniform scaling
            a.y.total_cmp(&b.y)
        })
        .cloned())
}

/// Finds the bigest font that can draw the string such that it's no larger than the given size.
pub fn fit_str_to_size(
    s: &str,
    destination: &Size,
    font: &rusttype::Font,
) -> Result<rusttype::Scale> {
    // the initial scale can't be too small or rusttype appears to have problems laying out glyphs, so pick an arbitrary default size
    let initial_scale = 12.0;
    // measure the string at that initial scale
    let text_bounds = measure_text(font, rusttype::Scale::uniform(initial_scale as f32), s)?;
    // find how much we'd have to adjust the starting size by to fit the measured string into the desired rectangle
    // take whichever scaling factor is smaller, so it doesn't extend past the rectangle bounds on the other axis
    let new_scale = (initial_scale * destination.width() / text_bounds.size().width())
        .min(initial_scale * destination.height() / text_bounds.size().height());
    Ok(rusttype::Scale::uniform(new_scale as f32))
}

fn measure_text(font: &rusttype::Font, scale: rusttype::Scale, s: &str) -> Result<Rectangle> {
    let mut result: Option<Rectangle> = None;
    for glyph in font.layout(s, scale, rusttype::Point { x: 0.0, y: 0.0 }) {
        if let Some(bounds) = glyph.pixel_bounding_box() {
            let bounds = Rectangle::from_two_points(
                &Point {
                    x: bounds.min.x as f64,
                    y: bounds.min.y as f64,
                },
                &Point {
                    x: bounds.max.x as f64,
                    y: bounds.max.y as f64,
                },
            );
            result = match result {
                Some(current) => Some(Rectangle::from_points(
                    [*current.min(), current.max(), *bounds.min(), bounds.max()].iter(),
                )?),
                None => Some(bounds),
            };
        }
    }
    Ok(result.ok_or("empty bounding rectangle".to_string())?)
}
