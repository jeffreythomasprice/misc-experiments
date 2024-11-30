use super::{colors::U8RGBA, texture::Texture};
use crate::{
    error::Error,
    math::{rect::Rect, size::Size, vec2::Vec2},
};
use log::*;
use rusttype::{gpu_cache::Cache, Font, PositionedGlyph, Scale};
use std::rc::Rc;
use web_sys::WebGl2RenderingContext;

pub struct TextureFont<'a> {
    context: Rc<WebGl2RenderingContext>,
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
        let (texture, cache) = Self::create_texture_and_cache(
            context.clone(),
            // just guess as to a sensible initial size so we do less re-sizing of the underlying texture later
            Size { width: 256, height: 256 },
        )?;
        Ok(Self {
            context,
            scale: Scale::uniform(scale),
            font,
            cache,
            texture,
        })
    }

    pub fn get_texture(&mut self) -> &Texture {
        &self.texture
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
                bounds,
            }),
        }
    }

    pub fn update_cache<'b, I>(&mut self, glyphs: I) -> Result<(), Error>
    where
        'a: 'b,
        I: Iterator<Item = &'b PositionedGlyph<'a>>,
    {
        // TODO can we avoid making a clone of the input glyphs?
        let glyphs = glyphs.cloned().collect::<Vec<_>>();
        loop {
            for glyph in glyphs.iter() {
                self.cache.queue_glyph(0, glyph.clone());
            }
            let mut abort_error = None;
            let cache_error = self.cache.cache_queued(|rect, data| {
                if abort_error.is_some() {
                    return;
                }

                let rgba_data = data
                    .iter()
                    .map(|greyscale| U8RGBA {
                        red: 255,
                        green: 255,
                        blue: 255,
                        alpha: *greyscale,
                    })
                    .collect::<Vec<_>>();
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
            });
            return match (abort_error, cache_error) {
                (Some(e), _) => Err(format!("failed to render glyph into texture cache: {e:?}"))?,
                (_, Err(e)) => {
                    let current_texture_size = self.texture.size();
                    let current_cache_size: Size<u32> = self.cache.dimensions().into();
                    let max_texture_size = Texture::max_size(&self.context)?;
                    let next_size = Size {
                        width: (current_texture_size.width.max(current_cache_size.width) * 2)
                            .max(1)
                            .min(max_texture_size as u32),
                        height: (current_texture_size.height.max(current_cache_size.height) * 2)
                            .max(1)
                            .min(max_texture_size as u32),
                    };
                    trace!("current cache size is too small, current texture size = {}, current cache size = {}, max texture size = {}, error = {:?}, next size = {}", current_texture_size, current_cache_size, max_texture_size, e, next_size);
                    if next_size == *current_texture_size || next_size == current_cache_size {
                        Err(format!("failed to cache glyphs, but also can't make texture any bigger, current texture size = {}, current cache size = {}, max texture size = {}", current_texture_size, current_cache_size, max_texture_size))?;
                    }
                    let (texture, cache) = Self::create_texture_and_cache(self.context.clone(), next_size)
                        .map_err(|e| format!("error resizing texture font cache: {e:?}"))?;
                    self.texture = texture;
                    self.cache = cache;
                    continue;
                }
                (_, _) => Ok(()),
            };
        }
    }

    pub fn rect_for(&mut self, glyph: &PositionedGlyph<'_>) -> Result<Option<(Rect<f32>, Rect<i32>)>, Error> {
        match self.cache.rect_for(0, glyph) {
            Ok(Some((uv_rect, screen_rect))) => Ok(Some((uv_rect.into(), screen_rect.into()))),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("error rendering glyph {:?}, error={:?}", glyph, e))?,
        }
    }

    pub fn bind(&self) {
        self.texture.bind();
    }

    pub fn bind_none(&self) {
        self.texture.bind_none();
    }

    fn create_texture_and_cache(context: Rc<WebGl2RenderingContext>, size: Size<u32>) -> Result<(Texture, Cache<'a>), Error> {
        let cache = Cache::builder().dimensions(size.width, size.height).build();
        let texture = Texture::new_with_size(context, size).map_err(|e| format!("error making texture for font cache: {e:?}"))?;
        Ok((texture, cache))
    }
}
