use super::buffer::Buffer;

pub struct Mesh<T> {
    vertex_buffer: Buffer<T>,
    index_buffer: Buffer<u16>,
}

impl<T> Mesh<T> {
    pub fn new(vertex_buffer: Buffer<T>, index_buffer: Buffer<u16>) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer<T> {
        &self.vertex_buffer
    }

    pub fn vertex_buffer_mut(&mut self) -> &mut Buffer<T> {
        &mut self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer<u16> {
        &self.index_buffer
    }

    pub fn index_buffer_mut(&mut self) -> &mut Buffer<u16> {
        &mut self.index_buffer
    }
}
