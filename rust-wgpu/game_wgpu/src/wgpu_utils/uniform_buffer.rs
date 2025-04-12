use bytemuck::Pod;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};

use super::buffer::Buffer;

pub struct UniformBuffer<T> {
    buffer: Buffer<T>,
    bind_group: BindGroup,
}

impl<T: Pod> UniformBuffer<T> {
    pub fn new_init(device: &Device, initial_value: T, binding: u32) -> Self {
        let buffer = Buffer::new_init(
            device,
            None,
            &[initial_value],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout(device, binding),
            entries: &[BindGroupEntry {
                binding,
                resource: buffer.buffer().as_entire_binding(),
            }],
        });
        Self { buffer, bind_group }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn enqueue_update(&mut self, queue: &Queue, value: T) {
        self.buffer.enqueue_update(queue, 0, &[value]);
    }
}

pub fn bind_group_layout(device: &Device, binding: u32) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}
