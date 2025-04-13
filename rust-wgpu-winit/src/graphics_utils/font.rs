use image::{DynamicImage, ImageBuffer, LumaA};
use rusttype::{Point, Rect, Scale};

pub struct FontLayoutGlyph {
    pub image: DynamicImage,
    pub bounds: Rect<i32>,
    pub advance: f32,
}

pub struct Font<'font> {
    font: rusttype::Font<'font>,
}

impl<'font> Font<'font> {
    pub fn new(font: rusttype::Font<'font>) -> Self {
        Self { font }
    }

    pub fn render_char_to_image(&self, c: char, scale: f32) -> FontLayoutGlyph {
        if let Some(glyph) = self
            .font
            .layout(
                &format!("{}", c),
                Scale::uniform(scale),
                Point { x: 0.0, y: 0.0 },
            )
            .next()
        {
            if let Some(glyph_bounding_box) = glyph.pixel_bounding_box() {
                let mut image_buffer = ImageBuffer::from_pixel(
                    glyph_bounding_box.width() as u32 + 1,
                    glyph_bounding_box.height() as u32 + 1,
                    LumaA([0, 0]),
                );
                glyph.draw(|x, y, v| {
                    let x = (x as i32) + glyph_bounding_box.min.x;
                    let y = (y as i32) + glyph_bounding_box.min.y;
                    let y = -y - 1;
                    if x >= 0 && y >= 0 {
                        let x = x as u32;
                        let y = y as u32;
                        if x < image_buffer.width() && y < image_buffer.height() {
                            image_buffer.put_pixel(x, y, LumaA([255, (v * 255.0) as u8]));
                        }
                    }
                });
                FontLayoutGlyph {
                    image: DynamicImage::ImageLumaA8(image_buffer),
                    bounds: glyph_bounding_box,
                    advance: glyph.unpositioned().h_metrics().advance_width,
                }
            } else {
                FontLayoutGlyph {
                    image: DynamicImage::ImageLumaA8(ImageBuffer::from_pixel(1, 1, LumaA([0, 0]))),
                    bounds: Rect {
                        min: Point { x: 0, y: 0 },
                        max: Point { x: 1, y: 1 },
                    },
                    advance: glyph.unpositioned().h_metrics().advance_width,
                }
            }
        } else {
            FontLayoutGlyph {
                image: DynamicImage::ImageLumaA8(ImageBuffer::from_pixel(1, 1, LumaA([0, 0]))),
                bounds: Rect {
                    min: Point { x: 0, y: 0 },
                    max: Point { x: 1, y: 1 },
                },
                advance: 0.0,
            }
        }
    }
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
