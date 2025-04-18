use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bytemuck::Zeroable;
use color_eyre::eyre::{Result, eyre};
use glam::Vec2;
use rusttype::GlyphId;
use wgpu::{Device, Queue};

use crate::texture::{Texture, TextureBindings};

use super::{basic_types::Rect, font::Font, texture_atlas::TextureAtlas};

pub struct LayedOutGlyph {
    pub texture_coordinate_bounds: Rect,
    pub pixel_bounds: Rect,
}

pub struct LayoutPerTexture {
    pub texture: Arc<Texture>,
    pub glyphs: Vec<LayedOutGlyph>,
}

pub struct LayoutResults {
    pub total_pixel_bounds: Rect,
    pub layout: Vec<LayoutPerTexture>,
}

struct AtlasExtraData {
    id: GlyphId,
    image_bounds: rusttype::Rect<i32>,
    advance: f32,
}

impl AtlasExtraData {
    pub fn is_drawable(&self) -> bool {
        self.id.0 != 0
    }
}

pub struct TextureAtlasFont {
    device: Arc<Device>,
    queue: Arc<Queue>,
    bindings: TextureBindings,
    font: Arc<Font<'static>>,
    scale: f32,
    line_height: f32,
    all_chars: HashSet<char>,
    texture_atlas: TextureAtlas<char>,
    extra_data: HashMap<char, AtlasExtraData>,
}

impl TextureAtlasFont {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        font: Arc<Font<'static>>,
        scale: f32,
    ) -> Result<Self> {
        let (texture_atlas, extra_data) = Self::create_texture_atlas(
            device.clone(),
            queue.clone(),
            bindings,
            &font,
            scale,
            [].iter(),
        )?;
        let v_metrics = font.v_metrics(scale);
        Ok(Self {
            device,
            queue,
            bindings,
            font,
            scale,
            line_height: v_metrics.ascent.abs()
                + v_metrics.descent.abs()
                + v_metrics.line_gap.abs(),
            all_chars: HashSet::new(),
            texture_atlas,
            extra_data,
        })
    }

    pub fn layout(&mut self, s: &str) -> Result<LayoutResults> {
        // try to render it assuming the current atlas holds all characters
        if let Some(result) = self.try_layout(s)? {
            // success
            Ok(result)
        } else {
            // nope, rebuild the atlas to include the new stuff
            // all_chars should have been already filled in by try_layout
            let (texture_atlas, extra_data) = Self::create_texture_atlas(
                self.device.clone(),
                self.queue.clone(),
                self.bindings,
                &self.font,
                self.scale,
                self.all_chars.iter(),
            )?;
            self.texture_atlas = texture_atlas;
            self.extra_data = extra_data;
            self.try_layout(s)?.ok_or(eyre!("rebuilt atlas to include whole string, but still couldn't render the whole thing: {}", s))
        }
    }

    fn try_layout(&mut self, s: &str) -> Result<Option<LayoutResults>> {
        // tracking whether we need to rebuild the texture atlas or not
        let mut rebuild = false;

        let mut total_pixel_bounds = Rect::zeroed();
        let mut cursor = Vec2::zeroed();
        // capacity of 1 because we assume we have a single texture
        let mut layout = Vec::<LayoutPerTexture>::with_capacity(1);

        for c in s.chars() {
            rebuild |= self.all_chars.insert(c);
            // if we do, we can stop trying to place all the characters, we're going to have to start over with a new atlas anyway
            if rebuild {
                // but continue, because we want to make sure our full set of all possible characters includes this entire string
                continue;
            }

            // we can try to place this character from the atlas
            // get the data for this glyph
            let (texture, texture_coordinate_bounds) = self
                .texture_atlas
                .get(&c)
                .ok_or(eyre!("texture atlas should already contain: {c}"))?;
            let extra_data = self
                .extra_data
                .get(&c)
                .ok_or(eyre!("extra data should already contain: {c}"))?;
            let pixel_bounds = Rect::from_origin_size(
                Vec2::new(
                    extra_data.image_bounds.min.x as f32,
                    extra_data.image_bounds.min.y as f32,
                ) + cursor,
                Vec2::new(
                    extra_data.image_bounds.width() as f32,
                    extra_data.image_bounds.height() as f32,
                ),
            );

            // keep track of the total size
            total_pixel_bounds =
                Rect::bounding_box_around_two_rects(&total_pixel_bounds, &pixel_bounds);

            // next position in pixel space
            if c == '\n' {
                cursor.x = 0.0;
                cursor.y += self.line_height;
            } else {
                cursor.x += extra_data.advance;
            }

            // is this even drawable?
            if extra_data.is_drawable() {
                // find the layout for this texture, or make a new one
                let layout = if let Some(existing_layout) = layout
                    .iter_mut()
                    .find(|l| Arc::ptr_eq(&l.texture, &texture))
                {
                    existing_layout
                } else {
                    layout.push(LayoutPerTexture {
                        texture: texture.clone(),
                        glyphs: Vec::new(),
                    });
                    let i = layout.len() - 1;
                    &mut layout[i]
                };
                layout.glyphs.push(LayedOutGlyph {
                    texture_coordinate_bounds,
                    pixel_bounds,
                });
            }
        }
        Ok(if rebuild {
            None
        } else {
            Some(LayoutResults {
                total_pixel_bounds,
                layout,
            })
        })
    }

    fn create_texture_atlas<'a>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        font: &Font<'a>,
        scale: f32,
        chars: impl Iterator<Item = &'a char>,
    ) -> Result<(TextureAtlas<char>, HashMap<char, AtlasExtraData>)> {
        let mut images = HashMap::new();
        let mut image_bounds = HashMap::new();
        for c in chars {
            let glyph = font.render_char_to_image(*c, scale);
            images.insert(*c, glyph.image);
            image_bounds.insert(
                *c,
                AtlasExtraData {
                    id: glyph.id,
                    image_bounds: glyph.bounds,
                    advance: glyph.advance,
                },
            );
        }
        let texture_atlas = TextureAtlas::new(device, queue, bindings, images)?;
        Ok((texture_atlas, image_bounds))
    }
}
