use color_eyre::eyre::Result;

use crate::geom::{Affine2f32, Rectf32, Vec2u32};

use super::color::{RGBAf32, RGBAu8};

pub trait Texture {
    fn size(&self) -> Vec2u32;
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn update(&mut self, size: Vec2u32, pixels: &[RGBAu8], destination: Vec2u32) -> Result<()>;
}

pub struct Material<'a, Texture: self::Texture> {
    pub color: Option<RGBAf32>,
    pub texture: Option<&'a Texture>,
    pub blend: bool,
}

pub trait Renderer {
    type Texture: Texture;

    fn create_texture_size(&mut self, size: Vec2u32) -> Result<Self::Texture>;
    fn create_texture_pixels(
        &mut self,
        size: Vec2u32,
        pixels: &mut [RGBAu8],
    ) -> Result<Self::Texture>;

    fn clear_screen(&mut self, clear_color: RGBAf32) -> Result<()>;

    fn set_ortho(&mut self, r: Rectf32) -> Result<()>;

    fn fill_rect(
        &mut self,
        rect: Rectf32,
        transform: Affine2f32,
        material: &Material<'_, Self::Texture>,
    ) -> Result<()>;

    fn present() -> Result<()>;
}
