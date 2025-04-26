use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RGBAu8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RGBAf32 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl From<RGBAf32> for RGBAu8 {
    fn from(value: RGBAf32) -> Self {
        Self {
            red: (value.red * 255.0) as u8,
            green: (value.green * 255.0) as u8,
            blue: (value.blue * 255.0) as u8,
            alpha: (value.alpha * 255.0) as u8,
        }
    }
}

impl From<RGBAu8> for RGBAf32 {
    fn from(value: RGBAu8) -> Self {
        Self {
            red: value.red as f32 / 255.0,
            green: value.green as f32 / 255.0,
            blue: value.blue as f32 / 255.0,
            alpha: value.alpha as f32 / 255.0,
        }
    }
}
