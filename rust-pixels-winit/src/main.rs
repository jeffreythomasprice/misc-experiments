use std::{process::exit, sync::Arc};

use color_eyre::eyre::Result;
use pixels::{Pixels, SurfaceTexture};
use tracing::*;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

struct WindowState {
    window: Arc<Window>,
    pixels: Pixels<'static>,
}

impl WindowState {
    pub fn new(event_loop: &ActiveEventLoop, display_size: LogicalSize<u32>) -> Result<Self> {
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

        let surface_texture = SurfaceTexture::new(
            window_size.width as u32,
            window_size.height as u32,
            window.clone(),
        );
        let pixels = Pixels::new(display_size.width, display_size.height, surface_texture)?;

        Ok(Self { window, pixels })
    }
}

pub struct App {
    window_state: Option<WindowState>,
}

impl App {
    pub fn new() -> Self {
        Self { window_state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window_state.is_none() {
            match WindowState::new(event_loop, LogicalSize::new(1024, 768)) {
                Ok(window_state) => {
                    self.window_state.replace(window_state);
                }
                Err(e) => {
                    error!("failed to create window: {e:?}");
                    exit(1);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
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
                if let Some(window_state) = &mut self.window_state {
                    if let Err(e) = window_state
                        .pixels
                        .resize_surface(physical_size.width, physical_size.height)
                    {
                        error!("error resizing: {e:?}");
                        exit(1);
                    }
                }
            }

            winit::event::WindowEvent::RedrawRequested => {
                if let Some(window_state) = &mut self.window_state {
                    let width = window_state.pixels.texture().width();
                    let height = window_state.pixels.texture().height();
                    let pixels = window_state.pixels.frame_mut();
                    for y in 0..height {
                        let b = ((y as f64) / (height as f64) * 255.0) as u8;
                        for x in 0..width {
                            let a = ((x as f64) / (width as f64) * 255.0) as u8;
                            let i = (4 * (x + y * width)) as usize;
                            let pixel = &mut pixels[i..(i + 4)];
                            pixel[0] = a;
                            pixel[1] = b;
                            pixel[2] = a;
                            pixel[3] = 255;
                        }
                    }

                    if let Err(e) = window_state.pixels.render() {
                        error!("error rendering: {e:?}");
                        exit(1);
                    }
                    window_state.window.request_redraw();
                }
            }

            _ => (),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();

    color_eyre::install()?;

    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter("info,game=trace")
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut App::new())?;

    Ok(())
}
