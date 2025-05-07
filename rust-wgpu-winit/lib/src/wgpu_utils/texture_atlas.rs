use std::{collections::HashMap, hash::Hash, sync::Arc};

use color_eyre::eyre::{Result, eyre};
use glam::{UVec2, Vec2};
use image::DynamicImage;
use wgpu::{Device, Queue};

use super::texture::{Texture, TextureBindings};

use crate::basic_types::Rect;

struct TextureInfo {
    texture: Arc<Texture>,
    bounds: Rect,
}

pub struct TextureAtlas<Handle> {
    handles: HashMap<Handle, TextureInfo>,
}

impl<Handle> TextureAtlas<Handle>
where
    Handle: Clone + Eq + Hash,
{
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        images: HashMap<Handle, DynamicImage>,
    ) -> Result<Self> {
        let max_texture_size = device.limits().max_texture_dimension_2d;

        let mut next_id = 0;
        let mut id_to_handle = HashMap::new();
        let mut nodes = Vec::new();
        for (handle, image) in images.iter() {
            let id = next_id;
            next_id += 1;
            id_to_handle.insert(id, handle);

            nodes.push(binpack2d::Dimension::with_id(
                id,
                image.width() as i32,
                image.height() as i32,
                0,
            ));
        }

        let pack_results = binpack2d::pack_bins(
            binpack2d::BinType::MaxRects,
            &nodes,
            max_texture_size as i32,
            max_texture_size as i32,
            false,
        )?;

        let mut handles = HashMap::new();
        for bin in pack_results.iter() {
            // build the texture for this bin
            let bin_width = bin.width() as f32;
            let bin_height = bin.height() as f32;
            let mut texture = Texture::with_size(
                device.clone(),
                queue.clone(),
                bindings,
                bin.width() as u32,
                bin.height() as u32,
            )?;
            for r in bin.iter() {
                let id = r.id();
                let handle = *id_to_handle
                    .get(&id)
                    .ok_or(eyre!("failed to find id: {id}"))?;
                let image = images
                    .get(handle)
                    .ok_or(eyre!("failed to find image for id: {id}"))?;
                texture.enqueue_update(image, UVec2::new(r.x() as u32, r.y() as u32));
            }

            // now the texture is sharable, make each glyph using positions on that texture
            let texture = Arc::new(texture);
            for r in bin.iter() {
                let id = r.id();
                let handle = *id_to_handle
                    .get(&id)
                    .ok_or(eyre!("failed to find id: {id}"))?;
                let bounds = Rect::from_origin_size(
                    Vec2::new((r.x() as f32) / bin_width, (r.y() as f32) / bin_height),
                    Vec2::new(
                        (r.width() as f32) / bin_width,
                        (r.height() as f32) / bin_height,
                    ),
                );
                handles.insert(
                    handle.clone(),
                    TextureInfo {
                        texture: texture.clone(),
                        bounds,
                    },
                );
            }
        }

        Ok(Self { handles })
    }

    pub fn get(&self, handle: &Handle) -> Option<(Arc<Texture>, Rect)> {
        let result = self.handles.get(handle)?;
        Some((result.texture.clone(), result.bounds))
    }
}
