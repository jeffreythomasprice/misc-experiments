use bytemuck::Zeroable;

use super::Vec2f32;

#[derive(Debug, Clone, Copy)]
pub struct Affine2f32 {
    pub translation: Vec2f32,
    pub rotation: f32,
    pub scale: Vec2f32,
}

impl Affine2f32 {
    pub fn identity() -> Self {
        Self::with_translation_rotation_scale(Vec2f32::zeroed(), 0.0, Vec2f32::new(1.0, 1.0))
    }

    pub fn with_translation_rotation_scale(
        translation: Vec2f32,
        rotation: f32,
        scale: Vec2f32,
    ) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    // TODO into matrix3x3
    // TODO into matrix4x4
}
