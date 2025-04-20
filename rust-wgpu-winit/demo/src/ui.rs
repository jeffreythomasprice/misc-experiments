use std::sync::{Arc, Mutex};

use color_eyre::eyre::Result;
use lib::{
    basic_types::{Affine2, Rect, Vertex2DTextureCoordinateColor},
    colors::Color,
    mesh::Mesh,
    mesh_builder::MeshBuilder,
    pipelines::pipeline2d,
    texture::Texture,
    texture_atlas_font::{Alignment, TextureAtlasFont},
};
use wgpu::{BlendState, Device, Queue, RenderPass, SurfaceConfiguration};

pub struct Text {
    device: Arc<Device>,
    queue: Arc<Queue>,
    font: Arc<Mutex<TextureAtlasFont>>,
    text: String,
    alignment: Alignment,
    affine: Affine2,
    transform: pipeline2d::Transform,
    is_dirty: bool,
    meshes_and_textures: Vec<(Mesh<Vertex2DTextureCoordinateColor>, Arc<Texture>)>,
}

impl Text {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        font: Arc<Mutex<TextureAtlasFont>>,
        affine: Affine2,
        text: String,
        alignment: Alignment,
    ) -> Self {
        Self {
            device: device.clone(),
            queue,
            font,
            text,
            alignment,
            affine,
            transform: pipeline2d::Transform::new(&device, affine.into()),
            is_dirty: true,
            meshes_and_textures: Vec::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.is_dirty = true;
    }

    pub fn alignment(&self) -> &Alignment {
        &self.alignment
    }

    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
        self.is_dirty = true;
    }

    pub fn affine(&self) -> &Affine2 {
        &self.affine
    }

    pub fn set_affine(&mut self, affine: Affine2) {
        self.affine = affine;
        self.is_dirty = true;
    }

    pub fn render(&mut self, render_pass: &mut pipeline2d::RenderPass) -> Result<()> {
        if self.is_dirty {
            self.is_dirty = false;
            self.enqueue_updates()?;
        }

        for (mesh, texture) in self.meshes_and_textures.iter() {
            render_pass.draw(mesh, texture, &self.transform);
        }

        Ok(())
    }

    fn enqueue_updates(&mut self) -> Result<()> {
        /*
        TODO re-use existing meshes if possible instead of always creating new ones
        TODO use a single mesh with different offsets for each texture?
        */
        let mut font = self.font.lock().unwrap();
        let layout = font.layout(&self.text, &self.alignment)?;
        self.meshes_and_textures.clear();
        for per_texture in layout.layout.iter() {
            let mut mesh_builder = MeshBuilder::<Vertex2DTextureCoordinateColor>::new();
            for glyph in per_texture.glyphs.iter() {
                mesh_builder.rectangle(
                    glyph.pixel_bounds,
                    glyph.texture_coordinate_bounds,
                    Color::WHITE,
                );
            }
            let mesh = mesh_builder.create_mesh(&self.device);
            self.meshes_and_textures
                .push((mesh, per_texture.texture.clone()));
        }

        self.transform
            .enqueue_update(&self.queue, self.affine.into());

        Ok(())
    }
}
