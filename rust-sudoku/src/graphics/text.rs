use std::fmt::Display;
use web_sys::CanvasRenderingContext2d;

use crate::{
    dom::{create_canvas, get_context}, Result,
};

use super::{Point, Rectangle, Size};

#[derive(Debug, Clone, Copy)]
pub enum SizeUnits {
    Pixels,
}

impl Display for SizeUnits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeUnits::Pixels => write!(f, "px"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontSize {
    pub value: f64,
    pub units: SizeUnits,
}

impl FontSize {
    pub fn scaled_by(&self, scale: f64) -> FontSize {
        FontSize {
            value: self.value * scale,
            units: self.units,
        }
    }
}

impl Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.value, self.units)
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub family: String,
    pub size: FontSize,
}

impl Font {
    pub fn set_on_context(&self, context: &CanvasRenderingContext2d) {
        context.set_font(&format!("{self}"));
    }

    pub fn scaled_by(&self, scale: f64) -> Font {
        Font {
            family: self.family.clone(),
            size: self.size.scaled_by(scale),
        }
    }
}

impl Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.size, self.family)
    }
}

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

pub fn fill_string(
    context: &CanvasRenderingContext2d,
    s: &str,
    destination_bounds: &Rectangle,
    font: &Font,
    horizontal_align: HorizontalStringAlign,
    vertical_align: VerticalStringAlign,
) -> Result<()> {
    font.set_on_context(context);
    let text_bounds = measure_text(context, s)?;
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
            };

    context.fill_text(s, x, y)?;

    Ok(())
}

/// As fit_str_to_size but the biggest font such that none of the strings are larger than size.
pub fn fit_strings_to_size(
    strings: &Vec<String>,
    destination: &Size,
    font_family: &str,
) -> Result<Option<Font>> {
    let possible_results = strings
        .iter()
        .map(|s| fit_str_to_size(s, destination, font_family))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(possible_results
        .iter()
        .min_by(|a, b| a.size.value.total_cmp(&b.size.value))
        .cloned())
}

/// Finds the bigest font that can draw the string such that it's no larger than the given size.
pub fn fit_str_to_size(s: &str, destination: &Size, font_family: &str) -> Result<Font> {
    // create a test canvas on which we can measure stuff
    let canvas = create_canvas()?;
    canvas.set_width(0);
    canvas.set_height(0);
    let context = get_context(&canvas)?;
    // pick an arbitrary starting size
    let size = FontSize {
        value: 10.0,
        units: SizeUnits::Pixels,
    };
    let font = Font {
        family: font_family.to_string(),
        size,
    };
    font.set_on_context(&context);
    // measure how big that string would be when drawn at that size
    let text_bounds = measure_text(&context, s)?;
    // find how much we'd have to adjust the starting size by to fit the measured string into the desired rectangle
    // take whichever scaling factor is smaller, so it doesn't extend past the rectangle bounds on the other axis
    let new_font_size = (size.value * destination.width() / text_bounds.size().width())
        .min(size.value * destination.height() / text_bounds.size().height());
    // new font with that size
    let size = FontSize {
        value: new_font_size,
        units: size.units,
    };
    Ok(Font {
        family: font.family,
        size,
    })
}

fn measure_text(context: &CanvasRenderingContext2d, s: &str) -> Result<Rectangle> {
    let m = context.measure_text(s)?;
    Ok(Rectangle::from_origin_size(
        Point {
            x: m.actual_bounding_box_left(),
            y: m.actual_bounding_box_ascent(),
        },
        Size::new(
            m.actual_bounding_box_right() - m.actual_bounding_box_left(),
            m.actual_bounding_box_ascent() + m.actual_bounding_box_descent(),
        )?,
    ))
}
