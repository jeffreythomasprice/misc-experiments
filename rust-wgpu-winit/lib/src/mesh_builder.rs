use bytemuck::Pod;
use color_eyre::eyre::{Result, eyre};
use glam::Vec2;
use wgpu::{BufferAddress, BufferUsages, Device, Queue};

use crate::{buffer::Buffer, mesh::Mesh};

use super::{
    basic_types::{Rect, Vertex2DColor, Vertex2DTextureCoordinateColor},
    colors::Color,
};

pub struct MeshBuilder<T> {
    vertices: Vec<T>,
    indices: Vec<u16>,
    vertex_offset: u16,
    index_offset: u16,
}

impl<T> MeshBuilder<T> {
    pub fn new() -> Self {
        Self::new_with_offsets(0, 0)
    }

    pub fn new_with_offsets(vertex_offset: u16, index_offset: u16) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_offset,
            index_offset,
        }
    }

    pub fn triangle_fan(&mut self, vertices: &[T]) -> Result<&mut Self>
    where
        T: Clone,
    {
        if vertices.len() < 3 {
            Err(eyre!(
                "must provide at least 3 points to make a triangle fan"
            ))?;
        }
        let vertex_offset = self.vertex_offset + self.vertices.len() as u16;
        self.vertices.extend_from_slice(vertices);
        self.indices.reserve((vertices.len() - 2) * 3);
        for i in 1..(vertices.len() as u16 - 1) {
            let a = vertex_offset;
            let b = i + vertex_offset;
            let c = b + 1;
            self.indices.extend_from_slice(&[a, b, c]);
        }
        Ok(self)
    }

    pub fn triangle(&mut self, a: T, b: T, c: T) -> &mut Self
    where
        T: Clone,
    {
        self.triangle_fan(&[a, b, c]).unwrap()
    }

    pub fn quad(&mut self, a: T, b: T, c: T, d: T) -> &mut Self
    where
        T: Clone,
    {
        self.triangle_fan(&[a, b, c, d]).unwrap()
    }

    pub fn create_mesh(&self, device: &Device) -> Mesh<T>
    where
        T: Pod,
    {
        Mesh::new(
            Buffer::new_init(
                device,
                None,
                &self.vertices,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            ),
            Buffer::new_init(
                device,
                None,
                &self.indices,
                BufferUsages::INDEX | BufferUsages::COPY_DST,
            ),
        )
    }

    pub fn enqueue_update(&self, queue: &Queue, mesh: &mut Mesh<T>) -> &Self
    where
        T: Pod,
    {
        mesh.vertex_buffer_mut().enqueue_update(
            queue,
            (self.vertex_offset as BufferAddress) * (std::mem::size_of::<T>() as BufferAddress),
            &self.vertices,
        );
        mesh.index_buffer_mut().enqueue_update(
            queue,
            (self.index_offset as BufferAddress) * (std::mem::size_of::<u16>() as BufferAddress),
            &self.indices,
        );
        self
    }
}

impl MeshBuilder<Vertex2DColor> {
    pub fn rectangle(&mut self, bounds: Rect, color: Color) -> &mut Self {
        self.quad(
            Vertex2DColor {
                position: Vec2 {
                    x: bounds.min.x,
                    y: bounds.max.y,
                },
                color,
            },
            Vertex2DColor {
                position: Vec2 {
                    x: bounds.min.x,
                    y: bounds.min.y,
                },
                color,
            },
            Vertex2DColor {
                position: Vec2 {
                    x: bounds.max.x,
                    y: bounds.min.y,
                },
                color,
            },
            Vertex2DColor {
                position: Vec2 {
                    x: bounds.max.x,
                    y: bounds.max.y,
                },
                color,
            },
        )
    }
}

impl MeshBuilder<Vertex2DTextureCoordinateColor> {
    pub fn rectangle(&mut self, bounds: Rect, texture_bounds: Rect, color: Color) -> &mut Self {
        self.quad(
            Vertex2DTextureCoordinateColor {
                position: Vec2 {
                    x: bounds.min.x,
                    y: bounds.max.y,
                },
                texture_coordinate: Vec2 {
                    x: texture_bounds.min.x,
                    y: texture_bounds.min.y,
                },
                color,
            },
            Vertex2DTextureCoordinateColor {
                position: Vec2 {
                    x: bounds.min.x,
                    y: bounds.min.y,
                },
                texture_coordinate: Vec2 {
                    x: texture_bounds.min.x,
                    y: texture_bounds.max.y,
                },
                color,
            },
            Vertex2DTextureCoordinateColor {
                position: Vec2 {
                    x: bounds.max.x,
                    y: bounds.min.y,
                },
                texture_coordinate: Vec2 {
                    x: texture_bounds.max.x,
                    y: texture_bounds.max.y,
                },
                color,
            },
            Vertex2DTextureCoordinateColor {
                position: Vec2 {
                    x: bounds.max.x,
                    y: bounds.max.y,
                },
                texture_coordinate: Vec2 {
                    x: texture_bounds.max.x,
                    y: texture_bounds.min.y,
                },
                color,
            },
        )
    }
}
