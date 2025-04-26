use color_eyre::eyre::{Result, eyre};

use crate::geom::{Rectf32, Vec2f32};

pub struct Camera {
    // the size of the viewing area in it's coordinate system
    world_bounds: Rectf32,

    // the size of the window we're drawing to in it's coordinate system
    window_size: Vec2f32,

    min_scale: f32,
    max_scale: f32,

    // the point in world space that will be in the center of the window
    camera_position: Vec2f32,

    // (window space) * scale = (world space)
    scale: f32,
}

impl Camera {
    /// world_bounds = the size of the viewing area in it's coordinate system
    ///
    /// smallest_viewing_area = the size, in world coordinates, of the smallest the scaled window size can get
    ///
    /// window_size = the size of the window we're drawing to in it's coordinate system
    pub fn new(
        world_bounds: Rectf32,
        smallest_viewing_area: Vec2f32,
        window_size: Vec2f32,
    ) -> Result<Self> {
        if smallest_viewing_area.x > world_bounds.width()
            || smallest_viewing_area.y > world_bounds.height()
        {
            Err(eyre!(
                "smallest viewing area ({:?}) needs to be no bigger than the world size ({:?})",
                smallest_viewing_area,
                world_bounds
            ))?;
        }

        // we're going to start out centered
        let camera_position = world_bounds.center();

        /*
        when zoomed out as much as possible we're going to be showing the whole content letterboxed
        that means that either width or height of the scaled window size will be bigger than the world bounds, but the other one matches
        */
        let max_scale = find_scale_that_letterboxes(window_size, world_bounds.size());
        // min scale is the same, except we're letterboxing on the desired smallest viewing area
        let min_scale = find_scale_that_letterboxes(window_size, smallest_viewing_area);
        if min_scale > max_scale {
            Err(eyre!(
                "backwards scale clamps? {} should be <= {}",
                min_scale,
                max_scale
            ))?;
        }

        Ok(Self {
            world_bounds,
            window_size,
            min_scale,
            max_scale,
            camera_position,
            scale: max_scale,
        })
    }

    pub fn world_bounds(&self) -> &Rectf32 {
        &self.world_bounds
    }

    pub fn set_world_bounds(&mut self, r: Rectf32) {
        self.world_bounds = r;
        // TODO recalculate stuff
    }

    pub fn window_size(&self) -> &Vec2f32 {
        &self.window_size
    }

    pub fn set_window_size(&mut self, v: Vec2f32) {
        self.window_size = v;
        // TODO recalculate stuff
    }

    pub fn camera_position(&self) -> Vec2f32 {
        self.camera_position
    }

    pub fn set_camera_position(&mut self, new_pos: Vec2f32) {
        // TODO clamp such that we letterbox? have some different bounds that represents valid camera positions?
        self.camera_position.x = new_pos
            .x
            .clamp(self.world_bounds.min().x, self.world_bounds.max().x);
        self.camera_position.y = new_pos
            .y
            .clamp(self.world_bounds.min().y, self.world_bounds.max().y);
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.clamp(self.min_scale, self.max_scale);
    }

    pub fn pan(&mut self, delta: Vec2f32) {
        self.set_camera_position(self.camera_position + delta);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.scale = (self.scale + delta).clamp(self.min_scale, self.max_scale);
    }

    // the size of the area the window can see, in world coordinates
    pub fn view_bounds(&self) -> Rectf32 {
        let scaled_window_size = self.window_size * self.scale;
        let half_scaled_window_size = scaled_window_size * 0.5;
        Rectf32::with_corners(
            self.camera_position - half_scaled_window_size,
            self.camera_position + half_scaled_window_size,
        )
    }

    // TODO get ortho matrix
}

/// a and b are sizes
///
/// returns x such that `a * x = b` on one axis, and `a * x >= b` on the other
fn find_scale_that_letterboxes(a: Vec2f32, b: Vec2f32) -> f32 {
    let scale = b.x / a.x;
    if a.y * scale < b.y { b.y / a.y } else { scale }
}
