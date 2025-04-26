use bytemuck::{Pod, Zeroable};

use super::Vec2f32;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Rectf32 {
    min: Vec2f32,
    max: Vec2f32,
}

impl Rectf32 {
    pub fn with_origin_size(origin: Vec2f32, size: Vec2f32) -> Self {
        Self::with_corners(origin, origin + size)
    }

    pub fn with_corners(a: Vec2f32, b: Vec2f32) -> Self {
        let (x1, x2) = if a.x < b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (y1, y2) = if a.y < b.y { (a.y, b.y) } else { (b.y, a.y) };
        Self {
            min: Vec2f32 { x: x1, y: y1 },
            max: Vec2f32 { x: x2, y: y2 },
        }
    }

    pub fn min(&self) -> &Vec2f32 {
        &self.min
    }

    pub fn max(&self) -> &Vec2f32 {
        &self.max
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.y
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Vec2f32 {
        Vec2f32::new(self.width(), self.height())
    }

    pub fn center(&self) -> Vec2f32 {
        (self.min + self.max) / 2.0
    }
}
