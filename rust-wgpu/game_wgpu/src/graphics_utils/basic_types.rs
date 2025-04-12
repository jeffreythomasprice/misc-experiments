use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode, vertex_attr_array};

pub trait HasVertexBufferLayout {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl RGBA {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex2DRGBA {
    pub position: Vec2,
    pub color: RGBA,
}

impl HasVertexBufferLayout for Vertex2DRGBA {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x2, 1 => Float32x4];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex2DTextureCoordinateRGBA {
    pub position: Vec2,
    pub texture_coordinate: Vec2,
    pub color: RGBA,
}

impl HasVertexBufferLayout for Vertex2DTextureCoordinateRGBA {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: [VertexAttribute; 3] =
            vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Affine2(glam::Affine2);

impl From<glam::Affine2> for Affine2 {
    fn from(value: glam::Affine2) -> Self {
        Self(value)
    }
}

impl From<Affine2> for glam::Affine2 {
    fn from(val: Affine2) -> Self {
        val.0
    }
}

impl From<Affine2> for glam::Mat4 {
    fn from(val: Affine2) -> Self {
        Mat4::from_cols(
            Vec4::new(val.0.matrix2.x_axis.x, val.0.matrix2.x_axis.y, 0.0, 0.0),
            Vec4::new(val.0.matrix2.y_axis.x, val.0.matrix2.y_axis.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(val.0.translation.x, val.0.translation.y, 0.0, 1.0),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self {
        let p1 = origin;
        let p2 = origin + size;
        Self {
            min: Vec2::new(p1.x.min(p2.x), p1.y.min(p2.y)),
            max: Vec2::new(p1.x.max(p2.x), p1.y.max(p2.y)),
        }
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn width(&self) -> f32 {
        self.size().x
    }

    pub fn height(&self) -> f32 {
        self.size().y
    }
}
