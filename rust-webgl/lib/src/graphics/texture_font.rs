use std::rc::Rc;

use rusttype::{gpu_cache::Cache, Font, PositionedGlyph, Scale};
use web_sys::WebGl2RenderingContext;

use crate::{
    error::Error,
    math::{rect::Rect, size::Size, vec2::Vec2},
};

use super::{colors::U8RGBA, texture::Texture};

pub struct TextureFont<'a> {
    scale: Scale,
    font: Font<'a>,
    cache: Cache<'a>,
    texture: Texture,
}

#[derive(Debug)]
pub struct LayoutResult<'a> {
    pub glyphs: Vec<PositionedGlyph<'a>>,
    pub bounds: Rect<i32>,
}

impl<'a> TextureFont<'a> {
    pub fn new_with_bytes_and_scale(context: Rc<WebGl2RenderingContext>, bytes: &'a [u8], scale: f32) -> Result<Self, Error> {
        let font = Font::try_from_bytes(bytes).ok_or("error loading font")?;

        // TODO how to figure out texture size?
        let size = Size { width: 1024, height: 1024 };

        let cache = Cache::builder().dimensions(size.width, size.height).build();

        let texture = Texture::new_with_size(context, size).map_err(|e| format!("error making texture for font cache: {e:?}"))?;

        Ok(Self {
            scale: Scale::uniform(scale),
            font,
            cache,
            texture,
        })
    }

    pub fn layout(&self, text: &str) -> Option<LayoutResult<'a>> {
        let v_metrics = self.font.v_metrics(self.scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = rusttype::point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;

        let mut result_glyphs = Vec::new();
        let mut result_bounds = None;
        for c in text.chars() {
            if c.is_control() {
                match c {
                    '\r' => {
                        caret = rusttype::point(0.0, caret.y + advance_height);
                    }
                    '\n' => {}
                    _ => {}
                }
                continue;
            }
            let base_glyph = self.font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += self.font.pair_kerning(self.scale, id, base_glyph.id());
            }
            last_glyph_id = Some(base_glyph.id());
            let glyph = base_glyph.scaled(self.scale).positioned(caret);
            // TODO word wrapping?
            // if let Some(bb) = glyph.pixel_bounding_box() {
            //     if bb.max.x > width as i32 {
            //         caret = point(0.0, caret.y + advance_height);
            //         glyph.set_position(caret);
            //         last_glyph_id = None;
            //     }
            // }
            caret.x += glyph.unpositioned().h_metrics().advance_width;

            if let Some(glyph_bounds) = glyph.pixel_bounding_box() {
                result_bounds = match result_bounds {
                    Some(result_bounds) => Some(Rect::union(&result_bounds, &glyph_bounds.into())),
                    None => Some(glyph_bounds.into()),
                };
            }

            result_glyphs.push(glyph);
        }

        match (result_glyphs.as_slice(), result_bounds) {
            // no results, no bounding box
            ([], _) | (_, None) => None,
            // we have some results
            (_, Some(bounds)) => Some(LayoutResult {
                glyphs: result_glyphs,
                bounds: bounds,
            }),
        }
    }

    pub fn update_cache<I>(&mut self, glyphs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = PositionedGlyph<'a>>,
    {
        for glyph in glyphs {
            self.cache.queue_glyph(0, glyph);
        }
        let mut abort_error = None;
        self.cache
            .cache_queued(|rect, data| {
                if abort_error.is_some() {
                    return;
                }

                let mut rgba_data = Vec::with_capacity(data.len());
                for greyscale in data {
                    rgba_data.push(U8RGBA {
                        red: *greyscale,
                        green: *greyscale,
                        blue: *greyscale,
                        alpha: 255,
                    });
                }
                let size = Size {
                    width: rect.width(),
                    height: rect.height(),
                };

                if let Err(e) = self.texture.copy_pixels(
                    &rect.into(),
                    &Rect::with_position_and_size(Vec2 { x: 0, y: 0 }, size),
                    size,
                    &rgba_data,
                ) {
                    abort_error = Some(e);
                }
            })
            .map_err(|e| format!("error updating font cache: {e:?}"))?;
        if let Some(e) = abort_error {
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn bind(&self) {
        self.texture.bind();
    }

    pub fn bind_none(&self) {
        self.texture.bind_none();
    }
}
