use std::marker::PhantomData;

use bytemuck::Pod;
use wgpu::{
    BufferAddress, BufferUsages, Device, Label, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct Buffer<T> {
    buffer: wgpu::Buffer,
    len: usize,
    phantom: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    pub fn new_init(device: &Device, label: Label, contents: &[T], usage: BufferUsages) -> Self {
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(contents),
                usage,
            }),
            len: contents.len(),
            phantom: PhantomData {},
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn enqueue_update(&mut self, queue: &Queue, offset: BufferAddress, data: &[T]) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data));
    }
}
