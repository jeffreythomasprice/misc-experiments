mod app;
mod shaders;
mod texture;

use std::{pin::Pin, sync::Arc};

use anyhow::{Result, anyhow};
use glam::{Vec2, Vec4};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        allocator::{CommandBufferAllocator, StandardCommandBufferAllocator},
    },
    device::{Device, Queue},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        GraphicsPipeline, PipelineCreateFlags, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, RenderPass, Subpass},
    shader::ShaderModule,
};

use crate::{
    app::{App, EventHandler, EventHandlerInitOptions},
    shaders::{ShaderType, compile_shader},
    texture::Texture,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vertex2d {
    #[format(R32G32_SFLOAT)]
    position: Vec2,
    #[format(R32G32B32A32_SFLOAT)]
    color: Vec4,
}

struct Demo {
    vertex_buffer: Subbuffer<[Vertex2d]>,
    index_buffer: Subbuffer<[u16]>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    graphics_pipeline: Arc<GraphicsPipeline>,
}

impl Demo {
    fn new(
        EventHandlerInitOptions {
            physical_device,
            device,
            graphics_queue,
            render_pass,
            viewport,
            command_buffer_allocator,
        }: EventHandlerInitOptions,
    ) -> Result<Self> {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![
                Vertex2d {
                    position: Vec2::new(0.5, -0.5),
                    color: Vec4::new(1.0, 0.0, 1.0, 1.0),
                },
                Vertex2d {
                    position: Vec2::new(0.5, 0.5),
                    color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                },
                Vertex2d {
                    position: Vec2::new(-0.5, 0.5),
                    color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                },
                Vertex2d {
                    position: Vec2::new(-0.5, -0.5),
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                },
            ],
        )?;

        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![0u16, 1, 2, 2, 3, 0],
        )?;

        let vertex_shader = compile_shader(
            device.clone(),
            ShaderType::Vertex,
            include_str!("shader.vert"),
        )?;

        let fragment_shader = compile_shader(
            device.clone(),
            ShaderType::Fragment,
            include_str!("shader.frag"),
        )?;

        let graphics_pipeline = create_graphics_pipeline::<Vertex2d>(
            device.clone(),
            vertex_shader.clone(),
            fragment_shader.clone(),
            render_pass.clone(),
            viewport.clone(),
        )?;

        // TODO do something with texture
        let texture = Texture::new_from_image(
            physical_device.clone(),
            device.clone(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            graphics_queue.clone(),
            image::ImageReader::open("assets/ChatGPT Image Jan 15, 2026, 02_09_46 PM.png")?
                .decode()?,
        )?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
            vertex_shader,
            fragment_shader,
            graphics_pipeline,
        })
    }
}

impl EventHandler for Demo {
    fn recreate(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> Result<()> {
        self.graphics_pipeline = create_graphics_pipeline::<Vertex2d>(
            device.clone(),
            self.vertex_shader.clone(),
            self.fragment_shader.clone(),
            render_pass.clone(),
            viewport,
        )?;
        Ok(())
    }

    fn create_command_buffer(
        &mut self,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        graphics_queue: &Queue,
        framebuffer: Arc<Framebuffer>,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>> {
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            graphics_queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )?;

        unsafe {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.25, 0.5, 1.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )?
                .bind_pipeline_graphics(self.graphics_pipeline.clone())?
                .bind_vertex_buffers(0, self.vertex_buffer.clone())?
                .bind_index_buffer(self.index_buffer.clone())?
                .draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0)?
                .end_render_pass(Default::default())?;
        }

        Ok(builder.build()?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    App::run(Box::new(Demo::new))?;

    Ok(())
}

fn create_graphics_pipeline<VertexType>(
    device: Arc<Device>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Result<Arc<GraphicsPipeline>>
where
    VertexType: Vertex,
{
    let vertex_shader_entry_point = vertex_shader
        .entry_point("main")
        .ok_or(anyhow!("failed to get vertex shader entry point"))?;
    let fragment_shader_entry_point = fragment_shader
        .entry_point("main")
        .ok_or(anyhow!("failed to get fragment shader entry point"))?;

    let vertex_input_state = VertexType::per_vertex()
        .definition(&vertex_shader_entry_point)
        .map_err(|e| anyhow!("failed to get vertex input definition: {e:?}"))?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vertex_shader_entry_point),
        PipelineShaderStageCreateInfo::new(fragment_shader_entry_point),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())?,
    )
    .map_err(|e| anyhow!("failed to create pipeline layout: {e:?}"))?;

    let subpass =
        Subpass::from(render_pass.clone(), 0).ok_or(anyhow!("failed to create subpass"))?;

    Ok(GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                front_face: FrontFace::Clockwise,
                cull_mode: CullMode::Back,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )?)
}
