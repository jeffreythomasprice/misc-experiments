use std::sync::Arc;

use color_eyre::eyre::Result;
use glam::{UVec2, Vec2};
use image::DynamicImage;
use wgpu::{Device, Queue};

use crate::wgpu_utils::{
    font::Font,
    mesh::Mesh,
    texture::{Texture, TextureBindings},
};

use super::{
    basic_types::{Color, Rect, Vertex2DTextureCoordinateColor},
    mesh_builder::MeshBuilder,
};

pub struct Text {
    device: Arc<Device>,
    queue: Arc<Queue>,
    bindings: TextureBindings,
    font: Arc<Font<'static>>,
    scale: f32,
    mesh: Option<Mesh<Vertex2DTextureCoordinateColor>>,
    texture: Option<Texture>,
}

impl Text {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        font: Arc<Font<'static>>,
        scale: f32,
    ) -> Self {
        Self {
            device,
            queue,
            bindings,
            font,
            scale,
            mesh: None,
            texture: None,
        }
    }

    pub fn update(&mut self, s: &str) -> Result<()> {
        let (font_image, _) = self.font.render_to_new_image(s, self.scale);

        // update texture
        let font_image_width = font_image.width();
        let font_image_height = font_image.height();
        let (texture_width, texture_height) = if let Some(texture) = &mut self.texture {
            if font_image_width <= texture.width() && font_image_height <= texture.height() {
                // texture is big enough already, just copy the new image into it
                texture.enqueue_update(font_image, UVec2::ZERO);
                (texture.width(), texture.height())
            } else {
                // texture is too small to fit image, recreate
                let new_texture = self.create_texture(font_image)?;
                let result = (new_texture.width(), new_texture.height());
                self.texture.replace(new_texture);
                result
            }
        } else {
            // first time, create a new texture
            let new_texture = self.create_texture(font_image)?;
            let result = (new_texture.width(), new_texture.height());
            self.texture.replace(new_texture);
            result
        };

        // update mesh
        let mesh_builder = self.create_mesh_builder(
            font_image_width as f32,
            font_image_height as f32,
            texture_width as f32,
            texture_height as f32,
        );
        if let Some(mesh) = &mut self.mesh {
            // update existing mesh
            mesh_builder.enqueue_update(&self.queue, mesh);
        } else {
            // first time, create a new mesh
            self.mesh.replace(mesh_builder.create_mesh(&self.device));
        }

        Ok(())
    }

    pub fn get(&self) -> Option<(&Mesh<Vertex2DTextureCoordinateColor>, &Texture)> {
        match (&self.mesh, &self.texture) {
            (Some(mesh), Some(texture)) => Some((mesh, texture)),
            _ => None,
        }
    }

    fn create_texture(&self, font_image: DynamicImage) -> Result<Texture> {
        Texture::from_image(
            self.device.clone(),
            self.queue.clone(),
            self.bindings,
            font_image,
        )
    }

    fn create_mesh_builder(
        &self,
        font_image_width: f32,
        font_image_height: f32,
        texture_width: f32,
        texture_height: f32,
    ) -> MeshBuilder<Vertex2DTextureCoordinateColor> {
        let mut result = MeshBuilder::<Vertex2DTextureCoordinateColor>::new();
        result.rectangle(
            Rect::from_origin_size(
                Vec2::new(0.0, 0.0),
                Vec2::new(texture_width, texture_height),
            ),
            Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(
                    font_image_width / texture_width,
                    font_image_height / texture_height,
                ),
            },
            Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
        );
        return result;
    }
}
