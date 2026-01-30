use std::{process::exit, sync::Arc};

use anyhow::{Result, anyhow};
use tracing::*;
use vulkano::{
    Validated, VulkanError, VulkanLibrary,
    command_buffer::{
        CommandBufferExecFuture, PrimaryAutoCommandBuffer,
        allocator::{CommandBufferAllocator, StandardCommandBufferAllocator},
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    image::{Image, ImageLayout, ImageUsage, SampleCount, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    pipeline::graphics::viewport::Viewport,
    render_pass::{
        AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
        Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, SubpassDescription,
    },
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

pub trait EventHandler {
    fn recreate(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> Result<()>;
    fn create_command_buffer(
        &mut self,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        graphics_queue: &Queue,
        framebuffer: Arc<Framebuffer>,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>>;
}

pub type EventHandlerFactory<EH> =
    Box<dyn FnOnce(Arc<Device>, Arc<RenderPass>, Viewport) -> Result<EH>>;

struct AppState<EH> {
    event_handler: EH,

    window: Arc<Window>,

    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    viewport: Viewport,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

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

impl<EH> AppState<EH>
where
    EH: EventHandler,
{
    fn new(
        event_handler_factory: EventHandlerFactory<EH>,
        event_loop: &ActiveEventLoop,
    ) -> Result<Self> {
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

        let (physical_device, graphics_queue_family_index) = instance
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
                            && match physical_device
                                .surface_support(queue_family_index as u32, &surface) {
                                    Ok(x) => x,
                                    Err(e) => {
                                        trace!("failed to check surface support for potential physical device {:#?}: {:?}", physical_device, e);
                                        false
                                    }
                                }
                    })
                    .map(|graphics_queue_family_index| (physical_device, graphics_queue_family_index as u32))
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

        let (device, queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: graphics_queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let graphics_queue = match queues.collect::<Vec<_>>().as_slice() {
            [graphics_queue] => graphics_queue.clone(),
            _ => Err(anyhow!("mismatched number of queues"))?,
        };

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

        let render_pass = create_render_pass(device.clone(), &swapchain)?;

        let framebuffers = create_framebuffers(&images, render_pass.clone())?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let event_handler =
            event_handler_factory(device.clone(), render_pass.clone(), viewport.clone())?;

        Ok(Self {
            event_handler,

            window,

            device,
            graphics_queue,
            swapchain,
            render_pass,
            framebuffers,
            viewport,
            command_buffer_allocator,

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
            self.framebuffers = create_framebuffers(&new_images, self.render_pass.clone())?;

            if self.window_resized {
                self.window_resized = false;

                self.viewport.extent = new_dimensions.into();

                self.event_handler.recreate(
                    self.device.clone(),
                    self.render_pass.clone(),
                    self.viewport.clone(),
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

        if let Some(image_fence) = &self.fences[image_index as usize] {
            image_fence
                .wait(None)
                .map_err(|e| anyhow!("failed to wait for fence: {e}"))?;
        }

        let previous_future = match self.fences[self.previous_fence_index].clone() {
            None => {
                let mut now = sync::now(self.device.clone());
                now.cleanup_finished();
                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };

        let future = previous_future
            .join(acquire_future)
            .then_execute(
                self.graphics_queue.clone(),
                self.event_handler.create_command_buffer(
                    self.command_buffer_allocator.clone(),
                    &self.graphics_queue,
                    self.framebuffers[image_index as usize].clone(),
                )?,
            )
            .map_err(|e| anyhow!("future execute error: {e:?}"))?
            .then_swapchain_present(
                self.graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        self.fences[image_index as usize] = match future {
            Ok(value) => Some(Arc::new(value)),
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                self.recreate_swapchain = true;
                None
            }
            Err(Validated::Error(e)) => {
                println!("failed to flush future: {e}");
                None
            }
            Err(Validated::ValidationError(e)) => {
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

pub struct App<EH> {
    state: Option<AppState<EH>>,
    event_handler_factory: Option<EventHandlerFactory<EH>>,
}

impl<EH> App<EH>
where
    EH: EventHandler,
{
    pub fn run(event_handler_factory: EventHandlerFactory<EH>) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app = Self {
            state: None,
            event_handler_factory: Some(event_handler_factory),
        };
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

impl<EH> ApplicationHandler for App<EH>
where
    EH: EventHandler,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            info!("initializing vulkan");
            match self.event_handler_factory.take() {
                Some(event_handler_factory) => {
                    match AppState::new(event_handler_factory, event_loop) {
                        Ok(state) => self.state = Some(state),
                        Err(e) => {
                            error!("failed to initialize vulkan: {}", e);
                            exit(1);
                        }
                    };
                }
                None => {
                    error!(
                        "initialzing window state, but event handler is missing, did we init twice?"
                    );
                    exit(1);
                }
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            // TODO event handling should go to event handler, window close and keyboard
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
                    && let Err(e) = state.redraw_requested()
                {
                    error!("redraw request failed: {}", e);
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state
                    && let Err(e) = state.resize(physical_size)
                {
                    error!("resize failed: {}", e);
                }
            }
            _ => (),
        };
    }
}

fn create_render_pass(device: Arc<Device>, swapchain: &Swapchain) -> Result<Arc<RenderPass>> {
    RenderPass::new(
        device,
        RenderPassCreateInfo {
            attachments: vec![AttachmentDescription {
                format: swapchain.image_format(),
                samples: SampleCount::Sample1,
                load_op: AttachmentLoadOp::Clear,
                store_op: AttachmentStoreOp::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::PresentSrc,
                ..Default::default()
            }],
            subpasses: vec![SubpassDescription {
                color_attachments: vec![Some(AttachmentReference {
                    attachment: 0,
                    layout: ImageLayout::ColorAttachmentOptimal,
                    ..Default::default()
                })],
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .map_err(|e| anyhow!("failed to create render pass: {}", e))
}

fn create_framebuffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>> {
    images
        .iter()
        .map::<Result<Arc<Framebuffer>>, _>(|image| {
            let view = ImageView::new_default(image.clone())
                .map_err(|e| anyhow!("failed to create image view: {}", e))?;
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow!("failed to create framebuffer: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()
}
