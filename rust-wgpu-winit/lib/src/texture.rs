use std::sync::Arc;

use color_eyre::eyre::Result;
use glam::UVec2;
use image::DynamicImage;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, Origin3d, Queue, SamplerBindingType, SamplerDescriptor, ShaderStages,
    TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension,
};

#[derive(Debug, Clone, Copy)]
pub struct TextureBindings {
    pub texture: u32,
    pub sampler: u32,
}

pub struct Texture {
    device: Arc<Device>,
    queue: Arc<Queue>,
    binding: TextureBindings,
    size: Extent3d,
    texture: wgpu::Texture,
    bind_group: BindGroup,
}

impl Texture {
    pub fn with_size(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        Self::with_init_callback(device, queue, bindings, width, height, |_, _, _| Ok(()))
    }

    pub fn from_image(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        image: &DynamicImage,
    ) -> Result<Self> {
        let image_rgba8 = match image.as_rgba8() {
            Some(x) => x,
            None => &image.to_rgba8(),
        };
        let width = image.width();
        let height = image.height();
        Self::with_init_callback(
            device,
            queue,
            bindings,
            width,
            height,
            |queue, texture, size| {
                queue.write_texture(
                    TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    image_rgba8,
                    TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * width),
                        rows_per_image: Some(height),
                    },
                    size,
                );
                Ok(())
            },
        )
    }

    pub fn size(&self) -> &Extent3d {
        &self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn enqueue_update(&mut self, image: &DynamicImage, origin: UVec2) {
        let image_rgba8 = match image.as_rgba8() {
            Some(x) => x,
            None => &image.to_rgba8(),
        };
        let image_width = image.width();
        let image_height = image.height();
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: origin.x,
                    y: origin.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            image_rgba8,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_width),
                rows_per_image: Some(image_height),
            },
            Extent3d {
                width: image_width,
                height: image_height,
                depth_or_array_layers: 1,
            },
        );
        self.bind_group = Self::create_bind_group_layout(
            &self.device,
            self.binding.texture,
            self.binding.sampler,
            &self.texture,
        );
    }

    fn with_init_callback(
        device: Arc<Device>,
        queue: Arc<Queue>,
        bindings: TextureBindings,
        width: u32,
        height: u32,
        f: impl FnOnce(&Queue, &wgpu::Texture, Extent3d) -> Result<()>,
    ) -> Result<Self> {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        f(&queue, &texture, size)?;
        let bind_group =
            Self::create_bind_group_layout(&device, bindings.texture, bindings.sampler, &texture);
        Ok(Self {
            device,
            queue,
            binding: bindings,
            size,
            texture,
            bind_group,
        })
    }

    fn create_bind_group_layout(
        device: &Device,
        texture_binding: u32,
        sampler_binding: u32,
        texture: &wgpu::Texture,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout(device, texture_binding, sampler_binding),
            entries: &[
                BindGroupEntry {
                    binding: texture_binding,
                    resource: BindingResource::TextureView(
                        &texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: sampler_binding,
                    resource: BindingResource::Sampler(&device.create_sampler(
                        &SamplerDescriptor {
                            address_mode_u: AddressMode::ClampToEdge,
                            address_mode_v: AddressMode::ClampToEdge,
                            address_mode_w: AddressMode::ClampToEdge,
                            mag_filter: FilterMode::Linear,
                            min_filter: FilterMode::Linear,
                            mipmap_filter: FilterMode::Nearest,
                            ..Default::default()
                        },
                    )),
                },
            ],
        })
    }
}

pub fn bind_group_layout(
    device: &Device,
    texture_binding: u32,
    sampler_binding: u32,
) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: texture_binding,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: sampler_binding,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}
