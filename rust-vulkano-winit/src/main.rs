use std::{process::exit, sync::Arc};

use anyhow::{Result, anyhow};
use tracing::*;
use vulkano::{
    Validated, VulkanError, VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        allocator::{CommandBufferAllocator, StandardCommandBufferAllocator},
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexDefinition,
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::ShaderModule,
    single_pass_renderpass,
    swapchain::{
        self, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
        SwapchainPresentInfo,
    },
    sync::{
        self, GpuFuture,
        future::{FenceSignalFuture, JoinFuture},
    },
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

/*
TODO some references

https://vulkano.rs/07-windowing/01-introduction.html

https://github.com/vulkano-rs/vulkano-book/blob/main/chapter-code/07-windowing/main.rs

https://docs.rs/winit/latest/winit/
*/

// TODO no unwrap

#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex)]
#[repr(C)]
struct Vertex {
    // TODO use a proper math lib with vector2 type
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

struct AppState {
    window: Arc<Window>,

    device: Arc<Device>,
    // TODO rename graphics_queue? do we care about other queues?
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    // TODO refactor so we don't have a command buffer pre-allocated all the time? keep graphics pipeline around?
    vertex_buffer: Subbuffer<[Vertex]>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,

    window_resized: bool,
    recreate_swapchain: bool,
    fences: Vec<
        Option<
            Arc<
                FenceSignalFuture<
                    PresentFuture<
                        CommandBufferExecFuture<
                            JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>,
                        >,
                    >,
                >,
            >,
        >,
    >,
    previous_fence_index: usize,
}

impl AppState {
    fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let library = VulkanLibrary::new()?;

        let surface_required_extensions = Surface::required_extensions(&event_loop)?;
        info!(
            "surface required extensions: {:#?}",
            surface_required_extensions
        );

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: surface_required_extensions,
                ..Default::default()
            },
        )?;

        let window = Arc::new(event_loop.create_window(Window::default_attributes())?);

        let surface = Surface::from_window(instance.clone(), window.clone())?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };
        info!("device extensions: {:#?}", device_extensions);

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|x| x.supported_extensions().contains(&device_extensions))
            .filter_map(|physical_device| {
                physical_device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(queue_family_index, queue_family_properties)| {
                        queue_family_properties
                            .queue_flags
                            .contains(QueueFlags::GRAPHICS)
                            && physical_device
                                .surface_support(queue_family_index as u32, &surface)
                                .unwrap_or(false)
                    })
                    .map(|queue_family_index| (physical_device, queue_family_index as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .ok_or(anyhow!("failed to find a physical device"))?;

        trace!(
            "physical device properties: {:#?}",
            physical_device.properties()
        );
        trace!(
            "physical device extensions: {:#?}",
            physical_device.supported_extensions()
        );

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let capabilities =
                physical_device.surface_capabilities(&surface, Default::default())?;
            info!("Surface capabilities: {capabilities:#?}");

            let window_size = window.inner_size();
            let composite_alpha = capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .ok_or(anyhow!("no supported composite alpha"))?;
            let (image_format, _) =
                physical_device.surface_formats(&surface, Default::default())?[0];
            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: capabilities.min_image_count,
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha,
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow!("error creating swapchain: {e:?}"))?
        };

        // TODO unmacro
        let render_pass = single_pass_renderpass!(
            device.clone(),
             attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )?;

        let framebuffers = get_framebuffers(&images, render_pass.clone())?;

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

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
                Vertex {
                    position: [-0.5, -0.5],
                },
                Vertex {
                    position: [0.5, -0.5],
                },
                Vertex {
                    position: [0.0, 0.5],
                },
            ],
        )?;

        // TOOD un-macro?
        mod vertex_shader_source {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
                    #version 450

                    layout(location = 0) in vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                "
            }
        }
        let vertex_shader = vertex_shader_source::load(device.clone())?;

        // TOOD un-macro?
        mod fragment_shader_source {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
                    #version 450

                    layout(location = 0) out vec4 f_color;

                    void main() {
                        f_color = vec4(1.0, 0.5, 0.25, 1.0);
                    }
                "
            }
        }
        let fragment_shader = fragment_shader_source::load(device.clone())?;

        let graphics_pipeline = get_pipeline(
            device.clone(),
            vertex_shader.clone(),
            fragment_shader.clone(),
            render_pass.clone(),
            viewport.clone(),
        )?;

        // TODO impl rest of it

        let command_buffers = get_command_buffers(
            command_buffer_allocator.clone(),
            &queue,
            &graphics_pipeline,
            &framebuffers,
            &vertex_buffer,
        )?;

        Ok(Self {
            window,

            device,
            queue,
            swapchain,
            render_pass,
            viewport,
            command_buffer_allocator,

            vertex_buffer,
            vertex_shader,
            fragment_shader,

            command_buffers,

            window_resized: false,
            recreate_swapchain: false,
            fences: vec![None; images.len()],
            previous_fence_index: 0,
        })
    }

    fn redraw_requested(&mut self) -> Result<()> {
        if self.window_resized || self.recreate_swapchain {
            self.recreate_swapchain = false;

            let new_dimensions = self.window.inner_size();

            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(),
                    ..self.swapchain.create_info()
                })
                .map_err(|e| anyhow!("failed to recreate swapchain: {e:?}"))?;

            self.swapchain = new_swapchain;
            let new_framebuffers = get_framebuffers(&new_images, self.render_pass.clone())?;

            if self.window_resized {
                self.window_resized = false;

                self.viewport.extent = new_dimensions.into();
                let new_pipeline = get_pipeline(
                    self.device.clone(),
                    self.vertex_shader.clone(),
                    self.fragment_shader.clone(),
                    self.render_pass.clone(),
                    self.viewport.clone(),
                )?;
                self.command_buffers = get_command_buffers(
                    self.command_buffer_allocator.clone(),
                    &self.queue,
                    &new_pipeline,
                    &new_framebuffers,
                    &self.vertex_buffer,
                )?;
            }
        }

        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(e) => Err(e)?,
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        // wait for the fence related to this image to finish (normally this would be the oldest fence)
        if let Some(image_fence) = &self.fences[image_index as usize] {
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences[self.previous_fence_index].clone() {
            // Create a NowFuture
            None => {
                let mut now = sync::now(self.device.clone());
                now.cleanup_finished();

                now.boxed()
            }
            // Use the existing FenceSignalFuture
            Some(fence) => fence.boxed(),
        };

        let future = previous_future
            .join(acquire_future)
            .then_execute(
                self.queue.clone(),
                self.command_buffers[image_index as usize].clone(),
            )
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        self.fences[image_index as usize] = match future.map_err(Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                None
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                None
            }
        };

        self.previous_fence_index = image_index as usize;

        self.window.request_redraw();
        Ok(())
    }

    fn resize(&mut self, _size: PhysicalSize<u32>) -> Result<()> {
        self.window_resized = true;
        Ok(())
    }
}

struct App {
    state: Option<AppState>,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self { state: None })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            info!("initializing vulkan");
            match AppState::new(event_loop) {
                Ok(state) => self.state = Some(state),
                Err(e) => {
                    error!("failed to initialize vulkan: {}", e);
                    exit(1);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("window close requested");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                info!("escape key released");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state
                    && let Err(e) = state.redraw_requested() {
                        error!("redraw request failed: {}", e);
                    }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state
                    && let Err(e) = state.resize(physical_size) {
                        error!("resize failed: {}", e);
                    }
            }
            _ => (),
        };
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = App::new()?;
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn get_framebuffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>> {
    Ok(images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}

// TODO make generic on vertex type
fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Result<Arc<GraphicsPipeline>> {
    let vs = vs
        .entry_point("main")
        .ok_or(anyhow!("failed to get vertex shader entry point"))?;
    let fs = fs
        .entry_point("main")
        .ok_or(anyhow!("failed to get fragment shader entry point"))?;

    let vertex_input_state =
        <Vertex as vulkano::pipeline::graphics::vertex_input::Vertex>::per_vertex()
            .definition(&vs)?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())?,
    )?;

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

fn get_command_buffers(
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &[Arc<Framebuffer>],
    vertex_buffer: &Subbuffer<[Vertex]>,
) -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator.clone(),
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )?;

            // TODO draw is unsafe
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
                    .bind_pipeline_graphics(pipeline.clone())?
                    .bind_vertex_buffers(0, vertex_buffer.clone())?
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)?
                    .end_render_pass(Default::default())?;
            }

            Ok(builder.build()?)
        })
        .collect()
}
