mod app;
mod shaders;

use std::sync::Arc;

use anyhow::{Result, anyhow};
use glam::{Vec2, Vec4};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        allocator::CommandBufferAllocator,
    },
    device::{
        Device, Queue,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{
        Framebuffer, RenderPass, Subpass,
    },
    shader::ShaderModule,
};

use crate::{
    app::{App, EventHandler},
    shaders::{ShaderType, compile_shader},
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vertex2d {
    #[format(R32G32_SFLOAT)]
    position: Vec2,
    #[format(R32G32B32A32_SFLOAT)]
    color: Vec4,
}

struct DemoState {
    vertex_buffer: Subbuffer<[Vertex2d]>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    graphics_pipeline: Arc<GraphicsPipeline>,
}

impl DemoState {
    fn new(device: Arc<Device>, render_pass: Arc<RenderPass>, viewport: Viewport) -> Result<Self> {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let vertex_buffer = Buffer::from_iter(
            memory_allocator,
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
                    position: Vec2::new(-0.5, 0.5),
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                },
                Vertex2d {
                    position: Vec2::new(0.5, 0.5),
                    color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                },
                Vertex2d {
                    position: Vec2::new(0.0, -0.5),
                    color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                },
            ],
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

        Ok(Self {
            vertex_buffer,
            vertex_shader,
            fragment_shader,
            graphics_pipeline,
        })
    }

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
                .draw(self.vertex_buffer.len() as u32, 1, 0, 0)?
                .end_render_pass(Default::default())?;
        }

        Ok(builder.build()?)
    }
}

struct Demo {
    state: Option<DemoState>,
}

impl Demo {
    fn new() -> Self {
        Self { state: None }
    }
}

impl EventHandler for Demo {
    fn init(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> Result<()> {
        self.state = Some(DemoState::new(device, render_pass, viewport)?);
        Ok(())
    }

    fn recreate(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> Result<()> {
        self.state
            .as_mut()
            .ok_or(anyhow!("expected inner state to be initialized"))?
            .recreate(device, render_pass, viewport)?;
        Ok(())
    }

    fn create_command_buffer(
        &mut self,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        graphics_queue: &Queue,
        framebuffer: Arc<Framebuffer>,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>> {
        self
            .state
            .as_mut()
            .ok_or(anyhow!("expected inner state to be initialized"))?
            .create_command_buffer(command_buffer_allocator, graphics_queue, framebuffer)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    App::run(Demo::new())?;

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
            rasterization_state: Some(RasterizationState::default()),
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
