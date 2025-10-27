use glam::{USizeVec2, usizevec2};

#[derive(Debug, Clone)]
pub struct USizeRect {
    origin: USizeVec2,
    size: USizeVec2,
}

impl USizeRect {
    pub fn new_origin_size(origin: USizeVec2, size: USizeVec2) -> Self {
        Self { origin, size }
    }

    pub fn new_corners(p1: USizeVec2, p2: USizeVec2) -> Self {
        let x1 = p1.x.min(p2.x);
        let x2 = p1.x.max(p2.x);
        let y1 = p1.y.min(p2.y);
        let y2 = p1.y.max(p2.y);
        let p1 = usizevec2(x1, y1);
        let p2 = usizevec2(x2, y2);
        Self {
            origin: p1,
            size: p2 - p1,
        }
    }

    pub fn origin(&self) -> &USizeVec2 {
        &self.origin
    }

    pub fn size(&self) -> &USizeVec2 {
        &self.size
    }

    pub fn width(&self) -> usize {
        self.size.x
    }

    pub fn height(&self) -> usize {
        self.size.y
    }

    pub fn min(&self) -> &USizeVec2 {
        &self.origin
    }

    pub fn max(&self) -> USizeVec2 {
        self.origin + self.size
    }

    pub fn contains(&self, p: &USizeVec2) -> bool {
        p.x >= self.min().x && p.x < self.max().x && p.y >= self.min().y && p.y < self.max().y
    }
}
