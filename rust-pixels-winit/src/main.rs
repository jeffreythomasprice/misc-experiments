use std::{process::exit, sync::Arc};

use color_eyre::eyre::{Result, eyre};
use pixels::{Pixels, SurfaceTexture};
use tiny_skia::{FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};
use tracing::*;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

const DESIRED_SIZE: LogicalSize<u32> = LogicalSize::new(1024, 768);

struct WindowState {
    window: Arc<Window>,
    pixels: Pixels<'static>,
}

impl WindowState {
    pub fn new(event_loop: &ActiveEventLoop, display_size: LogicalSize<u32>) -> Result<Self> {
        let window_size = PhysicalSize::new(DESIRED_SIZE.width, DESIRED_SIZE.height);
        let mut window_attributes = WindowAttributes::default()
            .with_title("Experiment")
            .with_inner_size(window_size);
        if let Some(monitor) = event_loop.primary_monitor() {
            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            window_attributes = window_attributes.with_position(PhysicalPosition::new(
                monitor_position.x + ((monitor_size.width - window_size.width) / 2) as i32,
                monitor_position.y + ((monitor_size.height - window_size.height) / 2) as i32,
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

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) -> Result<()> {
        self.pixels
            .resize_surface(physical_size.width, physical_size.height)?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let width = self.pixels.texture().width();
        let height = self.pixels.texture().height();
        let pixels = self.pixels.frame_mut();

        for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
            let x = i % (width as usize);
            let y = i / (width as usize);
            let a = ((x as f64) / (width as f64) * 255.0) as u8;
            let b = ((y as f64) / (height as f64) * 255.0) as u8;
            pixel.copy_from_slice(&[a, b, a, 255]);
        }

        let mut pixmap = PixmapMut::from_bytes(pixels, width, height)
            .ok_or(eyre!("error creating skia pixmap"))?;
        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 0, 0, 255);
        paint.anti_alias = true;
        pixmap.fill_path(
            &PathBuilder::from_circle((width as f32) * 0.25, (height as f32) * 0.5, 100.0)
                .ok_or(eyre!("error creating path"))?,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        pixmap.stroke_path(
            &PathBuilder::from_circle((width as f32) * 0.75, (height as f32) * 0.5, 100.0)
                .ok_or(eyre!("error creating path"))?,
            &paint,
            &Stroke {
                width: 5.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );

        self.pixels.render()?;
        self.window.request_redraw();

        Ok(())
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
            match WindowState::new(
                event_loop,
                LogicalSize::new(DESIRED_SIZE.width, DESIRED_SIZE.height),
            ) {
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
                    if let Err(e) = window_state.resize(physical_size) {
                        error!("error resizing: {e:?}");
                        exit(1);
                    }
                }
            }

            winit::event::WindowEvent::RedrawRequested => {
                if let Some(window_state) = &mut self.window_state {
                    if let Err(e) = window_state.render() {
                        error!("error rendering: {e:?}");
                        exit(1);
                    }
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
