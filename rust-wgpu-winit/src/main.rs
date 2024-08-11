use anyhow::{anyhow, Result};
use env_logger::DEFAULT_FILTER_ENV;
use log::*;
use std::{
    env, process,
    sync::{Arc, Mutex},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

struct GraphicsState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl GraphicsState {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        for adapter in instance.enumerate_adapters(wgpu::Backends::PRIMARY) {
            debug!("possible adapter: {:?}", adapter.get_info());
        }

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("failed to create adapter"))?;
        debug!("chosen adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None, // Trace path
            )
            .await
            .map_err(|e| anyhow!("error creating debug and queue: {e:?}"))?;

        let surface_caps = surface.get_capabilities(&adapter);
        debug!("surface capabilities: {:?}", surface_caps);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .ok_or_else(|| anyhow!("failed to find a surface format"))?;
        debug!("surface format: {:?}", surface_format);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size != self.size {
            self.config.width = size.width;
            self.config.height = size.height;
            self.size = size;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&self) -> Result<()> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                // TODO unneded?
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                // TODO unneded?
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.25,
                            g: 0.5,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    close_requested: bool,
    window: Option<Arc<Window>>,
    graphics_state: Arc<Mutex<Option<GraphicsState>>>,
}

impl App {
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<()> {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Experiment"))
                .map_err(|e| anyhow!("failed to create window: {e:?}"))?,
        );
        self.window = Some(window.clone());

        {
            let graphics_state_mutex = self.graphics_state.clone();
            let window = window.clone();
            tokio::spawn(async move {
                let graphics_state = match GraphicsState::new(window).await {
                    Ok(x) => x,
                    Err(e) => {
                        error!("failed to create graphics state: {e:?}");
                        process::exit(1);
                    }
                };

                let graphics_state_lock = &mut *graphics_state_mutex.lock().unwrap();
                graphics_state_lock.replace(graphics_state);
            });
        }

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Err(e) = self.init(event_loop) {
            error!("failed to initialize: {e:?}");
            process::exit(1);
        }
    }

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    // TODO unneeded?
                    // window.pre_present_notify();

                    if let Some(graphics_state) = self.graphics_state.lock().unwrap().as_ref() {
                        if let Err(e) = graphics_state.render() {
                            error!("error rendering: {e:?}");
                            process::exit(1);
                        }
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(graphics_state) = self.graphics_state.lock().unwrap().as_mut() {
                    graphics_state.resize(size);
                }
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                Key::Named(NamedKey::Escape) => {
                    self.close_requested = true;
                }
                _ => (),
            },
            _ => (),
        };
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.close_requested {
            event_loop.exit();
            return;
        }

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }

        // TODO if enough time has passed, update

        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(_) = env::var(DEFAULT_FILTER_ENV) {
        env::set_var(
            DEFAULT_FILTER_ENV,
            "trace, calloop=warn, wgpu_core=warn, wgpu_hal=warn",
        );
    }
    env_logger::builder().parse_default_env().init();

    let event_loop = EventLoop::new().map_err(|e| anyhow!("failed to create event loop: {e:?}"))?;

    event_loop
        .run_app(&mut App {
            close_requested: false,
            window: None,
            graphics_state: Arc::new(Mutex::new(None)),
        })
        .map_err(|e| anyhow!("event loop failed: {e:?}"))?;

    Ok(())
}
