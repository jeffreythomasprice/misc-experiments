use std::{
    process::exit,
    sync::Arc,
    time::{Duration, Instant},
};

use color_eyre::eyre::{Result, eyre};
use futures::executor::block_on;
use tracing::*;
use wgpu::{
    CompositeAlphaMode, Device, DeviceDescriptor, Instance, InstanceDescriptor, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

pub trait Renderer {
    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()>;
    fn render(&mut self, texture_view: TextureView) -> Result<()>;
    fn update(&mut self, duration: Duration) -> Result<()>;
}

struct WGPUInit {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_format: TextureFormat,
    surface_configuration: SurfaceConfiguration,
}

struct WindowState<R: Renderer> {
    window: Arc<Window>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Surface<'static>,
    surface_format: TextureFormat,
    surface_configuration: SurfaceConfiguration,
    renderer: R,
    last_render_time: Option<Instant>,
}

impl<R: Renderer> WindowState<R> {
    pub fn new(
        event_loop: &ActiveEventLoop,
        renderer_factory: impl Fn(Arc<Device>, Arc<Queue>, &SurfaceConfiguration) -> Result<R> + 'static,
    ) -> Result<Self> {
        let window_size = PhysicalSize::new(1024, 768);
        let mut window_attributes = WindowAttributes::default()
            .with_title("Experiment")
            .with_inner_size(window_size);
        if let Some(monitor) = event_loop.primary_monitor() {
            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            window_attributes = window_attributes.with_position(PhysicalPosition::new(
                monitor_position.x + ((monitor_size.width as i32) - window_size.width) / 2,
                monitor_position.y + ((monitor_size.height as i32) - window_size.height) / 2,
            ));
        }
        let window = Arc::new(event_loop.create_window(window_attributes)?);

        let WGPUInit {
            device,
            queue,
            surface,
            surface_format,
            surface_configuration,
        } = block_on(Self::init_wgpu(window.clone()))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let renderer = renderer_factory(device.clone(), queue.clone(), &surface_configuration)?;

        Ok(Self {
            window,
            device,
            queue,
            surface,
            surface_format,
            surface_configuration,
            renderer,
            last_render_time: None,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface
            .configure(&self.device, &self.surface_configuration);

        self.renderer.resize(size)?;

        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let surface_texture = self.surface.get_current_texture()?;
        let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
            format: Some(self.surface_format),
            ..Default::default()
        });
        self.renderer.render(texture_view)?;
        self.window.pre_present_notify();
        surface_texture.present();
        self.window.request_redraw();

        // TODO should do update in it's own thread with a fixed update rate
        let now = Instant::now();
        if let Some(last) = self.last_render_time {
            let duration = now.duration_since(last);
            self.renderer.update(duration)?;
        }
        self.last_render_time = Some(now);

        Ok(())
    }

    async fn init_wgpu(window: Arc<Window>) -> Result<WGPUInit> {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await?;
        debug!("adapter: {:?}", adapter);

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;
        debug!("device: {:?}", device);

        let size = window.inner_size();

        let surface = instance.create_surface(window)?;
        debug!("surface: {:?}", surface);

        let capabilities = surface.get_capabilities(&adapter);
        debug!("surface capabilities: {:?}", capabilities);
        let valid_formats = capabilities
            .formats
            .iter()
            .filter(|format| format.has_color_aspect())
            .collect::<Vec<_>>();
        trace!("valid formats: {:?}", valid_formats);
        // TODO pick a particular color format instead of just first?
        let surface_format = **valid_formats.first().ok_or(eyre!("no color formats"))?;
        debug!("format: {:?}", surface_format);

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![surface_format.add_srgb_suffix()],
        };
        surface.configure(&device, &surface_configuration);

        Ok(WGPUInit {
            device,
            queue,
            surface,
            surface_format,
            surface_configuration,
        })
    }
}

pub struct App<R: Renderer> {
    renderer_factory:
        Option<Box<dyn Fn(Arc<Device>, Arc<Queue>, &SurfaceConfiguration) -> Result<R>>>,
    window_state: Option<WindowState<R>>,
}

impl<R: Renderer> App<R> {
    pub fn new(
        renderer_factory: impl Fn(Arc<Device>, Arc<Queue>, &SurfaceConfiguration) -> Result<R> + 'static,
    ) -> Self {
        Self {
            renderer_factory: Some(Box::new(renderer_factory)),
            window_state: None,
        }
    }
}

impl<R: Renderer + 'static> ApplicationHandler for App<R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_state.is_none() {
            let renderer_factory = match self.renderer_factory.take() {
                Some(x) => x,
                None => {
                    error!("no renderer, did we init twice?");
                    exit(1);
                }
            };

            match WindowState::new(event_loop, renderer_factory) {
                Ok(x) => self.window_state.replace(x),
                Err(e) => {
                    error!("error intializing window: {e:?}");
                    exit(1);
                }
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // ignore other windows, if we ever have any
        if !if let Some(window_state) = &self.window_state {
            window_state.window.id() == window_id
        } else {
            false
        } {
            return;
        }

        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            winit::event::WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }

            winit::event::WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.window_state {
                    if let Err(e) = state.resize(physical_size) {
                        error!("error resizing: {e:?}");
                        exit(1);
                    }
                }
            }

            winit::event::WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.window_state {
                    if let Err(e) = state.render() {
                        error!("error rendering: {e:?}");
                        exit(1);
                    }
                }
            }

            _ => (),
        }
    }
}
