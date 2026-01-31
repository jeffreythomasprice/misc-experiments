use std::sync::Arc;

use anyhow::{Result, anyhow};
use image::{DynamicImage, EncodableLayout};
use vulkano::{
    NonExhaustive,
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
        PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract, RecordingCommandBuffer,
        allocator::CommandBufferAllocator, pool::CommandPool,
    },
    device::{Device, Queue, physical::PhysicalDevice},
    format::Format,
    image::{
        Image, ImageAspects, ImageCreateInfo, ImageLayout, ImageSubresourceRange, ImageTiling,
        ImageType, ImageUsage, SampleCount,
        sampler::{
            BorderColor, ComponentMapping, ComponentSwizzle, Filter, Sampler, SamplerAddressMode,
            SamplerCreateInfo, SamplerMipmapMode,
        },
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
    },
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    },
    pipeline::{PipelineShaderStageCreateFlags, graphics::depth_stencil::CompareOp},
    sync::{
        AccessFlags, GpuFuture, ImageMemoryBarrier, PipelineStage, Sharing,
        future::FenceSignalFuture,
    },
};

pub struct Texture {}

impl Texture {
    pub fn new_from_image(
        physical_device: Arc<PhysicalDevice>,
        device: Arc<Device>,
        memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        mut image: DynamicImage,
    ) -> Result<Self> {
        image.convert_color_space(
            image::metadata::Cicp::SRGB_LINEAR,
            Default::default(),
            image::ColorType::Rgba8,
        )?;

        let width = image.width();
        let height = image.height();

        let buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            image.into_bytes(),
        )?;

        let vk_image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                view_formats: vec![Format::R8G8B8A8_SRGB],
                extent: [width, height, 1],
                array_layers: 1,
                mip_levels: 1,
                samples: SampleCount::Sample1,
                tiling: ImageTiling::Optimal,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                sharing: Sharing::Exclusive,
                initial_layout: ImageLayout::Undefined,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )?;

        {
            let vk_image = vk_image.clone();
            one_time_submit(
                command_buffer_allocator.clone(),
                queue.clone(),
                |mut command_buffer_builder| {
                    let x = CopyBufferToImageInfo::buffer_image(buffer, vk_image);
                    command_buffer_builder
                        .copy_buffer_to_image(x)
                        .map_err(|e| anyhow!("failed to copy buffer to image: {e:?}"))?;
                    Ok(command_buffer_builder)
                },
            )?
            .wait(None)?;
        }

        let image_view = ImageView::new(
            vk_image.clone(),
            ImageViewCreateInfo {
                view_type: ImageViewType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                component_mapping: ComponentMapping {
                    r: ComponentSwizzle::Identity,
                    g: ComponentSwizzle::Identity,
                    b: ComponentSwizzle::Identity,
                    a: ComponentSwizzle::Identity,
                },
                subresource_range: ImageSubresourceRange {
                    aspects: ImageAspects::COLOR,
                    mip_levels: 0..1,
                    array_layers: 0..1,
                },
                usage: ImageUsage::SAMPLED,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow!("failed to create texture image view: {e:?}"))?;

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [
                    SamplerAddressMode::Repeat,
                    SamplerAddressMode::Repeat,
                    SamplerAddressMode::Repeat,
                ],
                /*
                TODO other example had anisotropy enabled, but get validation error:
                Requires one of:
                    device feature `sampler_anisotropy`
                */
                anisotropy: None,
                // anisotropy: Some(physical_device.properties().max_sampler_anisotropy),
                compare: Some(CompareOp::Always),
                border_color: BorderColor::IntOpaqueBlack,
                unnormalized_coordinates: false,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow!("failed to create texture sampler: {e:?}"))?;

        Ok(Self {})
    }
}

// TODO move me
fn one_time_submit(
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    queue: Arc<Queue>,
    f: impl FnOnce(
        AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
) -> Result<
    FenceSignalFuture<
        vulkano::command_buffer::CommandBufferExecFuture<vulkano::sync::future::NowFuture>,
    >,
> {
    let builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )?;

    let builder = f(builder)?;

    Ok(builder
        .build()?
        .execute(queue)?
        .then_signal_fence_and_flush()?)
}
