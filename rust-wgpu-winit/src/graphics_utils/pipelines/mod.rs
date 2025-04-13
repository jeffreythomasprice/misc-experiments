use std::sync::Arc;

use wgpu::{Device, Queue};

use crate::wgpu_utils::texture::TextureBindings;

pub mod pipeline_2d_textured;

pub trait HasDeviceAndQueue {
    fn device(&self) -> &Arc<Device>;
    fn queue(&self) -> &Arc<Queue>;
}

pub trait HasTextureBindings {
    fn texture_binding() -> TextureBindings;
}
