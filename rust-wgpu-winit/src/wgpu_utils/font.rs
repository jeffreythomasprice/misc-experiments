use image::{DynamicImage, ImageBuffer, LumaA};
use rusttype::{Point, Rect, Scale};

pub struct Font<'font> {
    font: rusttype::Font<'font>,
}

impl<'font> Font<'font> {
    pub fn new(font: rusttype::Font<'font>) -> Self {
        Self { font }
    }

    pub fn render_to_new_image(&self, s: &str, scale: f32) -> (DynamicImage, Rect<i32>) {
        let glyphs = self
            .font
            .layout(s, Scale::uniform(scale), Point { x: 0.0, y: 0.0 })
            .collect::<Vec<_>>();

        let bounding_box = glyphs
            .iter()
            .fold::<Option<Rect<i32>>, _>(None, |bbox, glyph| {
                match (bbox, glyph.pixel_bounding_box()) {
                    (Some(current), Some(this)) => Some(bounding_box_around_rects(&current, &this)),
                    (Some(current), None) => Some(current),
                    (None, Some(this)) => Some(this),
                    (None, None) => None,
                }
            });

        if let Some(bounding_box) = bounding_box {
            let mut image_buffer = ImageBuffer::from_pixel(
                bounding_box.width() as u32,
                bounding_box.height() as u32,
                LumaA([0, 0]),
            );
            for glyph in glyphs.iter() {
                if let Some(glyph_bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        let x = (x as i32) + glyph_bounding_box.min.x - bounding_box.min.x;
                        let y = (y as i32) + glyph_bounding_box.min.y - bounding_box.min.y;
                        let y = bounding_box.height() - y - 1;
                        if x >= 0 && y >= 0 {
                            let x = x as u32;
                            let y = y as u32;
                            if x < image_buffer.width() && y < image_buffer.height() {
                                image_buffer.put_pixel(x, y, LumaA([255, (v * 255.0) as u8]));
                            }
                        }
                    });
                }
            }
            (DynamicImage::ImageLumaA8(image_buffer), bounding_box)
        } else {
            (
                DynamicImage::ImageLumaA8(ImageBuffer::from_pixel(1, 1, LumaA([0, 0]))),
                Rect {
                    min: Point { x: 0, y: 0 },
                    max: Point { x: 0, y: 0 },
                },
            )
        }
    }

    // TODO create texture atlas
}

fn bounding_box_around_rects(a: &Rect<i32>, b: &Rect<i32>) -> Rect<i32> {
    Rect {
        min: Point {
            x: a.min.x.min(b.min.x),
            y: a.min.y.min(b.min.y),
        },
        max: Point {
            x: a.max.x.max(b.max.x),
            y: a.max.y.max(b.max.y),
        },
    }
}
