use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct U8RGBA {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct F32RGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}
