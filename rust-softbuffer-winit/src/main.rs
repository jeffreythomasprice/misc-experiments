use std::{num::NonZero, process::exit, sync::Arc};

use color_eyre::eyre::{Result, eyre};
use softbuffer::{Context, Surface};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};
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
    size: LogicalSize<NonZero<u32>>,
    window: Arc<Window>,
    surface: Surface<Arc<Window>, Arc<Window>>,
}

impl WindowState {
    pub fn new(event_loop: &ActiveEventLoop, size: LogicalSize<u32>) -> Result<Self> {
        let size: LogicalSize<NonZero<u32>> = match (size.width.try_into(), size.height.try_into())
        {
            (Ok(width), Ok(height)) => LogicalSize::new(width, height),
            _ => Err(eyre!(
                "must be at least one pixel in each dimension: {:?}",
                size
            ))?,
        };

        let window_size: PhysicalSize<u32> =
            PhysicalSize::new(size.width.into(), size.height.into());
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

        let context = Context::new(window.clone())
            .map_err(|e| eyre!("failed to make softbuffer context: {e:?}"))?;
        let mut surface = Surface::new(&context, window.clone())
            .map_err(|e| eyre!("failed to make softbuffer surface: {e:?}"))?;
        surface
            .resize(size.width, size.height)
            .map_err(|e| eyre!("failed to set initial size of softbuffer surface: {e:?}"))?;

        Ok(Self {
            size,
            window,
            surface,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        match (size.width.try_into(), size.height.try_into()) {
            (Ok(width), Ok(height)) => {
                self.surface
                    .resize(width, height)
                    .map_err(|e| eyre!("failed to resize softbuffer surface: {e:?}"))?;
                self.size = LogicalSize::new(width, height);
            }
            _ => (),
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| eyre!("failed to get surface buffer: {e:?}"))?;
        let pixels: &mut [u8] = bytemuck::try_cast_slice_mut(&mut buffer)
            .map_err(|e| eyre!("failed to cast buffer to u8 slice: {e:?}"))?;

        for (i, pixel) in pixels.chunks_mut(4).enumerate() {
            let x = i % (self.size.width.get() as usize);
            let y = i / (self.size.width.get() as usize);
            let a = ((x as f64) / (self.size.width.get() as f64) * 255.0) as u8;
            let b = ((y as f64) / (self.size.height.get() as f64) * 255.0) as u8;
            pixel[0] = 255 - a;
            pixel[1] = b;
            pixel[2] = a;
        }

        let mut pixmap =
            PixmapMut::from_bytes(pixels, self.size.width.get(), self.size.height.get())
                .ok_or(eyre!("error creating skia pixmap"))?;
        let mut paint = Paint::default();
        // tiny_skia assumes colors are bgra
        // softbuffer assumes colors are rgba
        paint.set_color_rgba8(0, 0, 255, 255);
        paint.anti_alias = true;
        pixmap.fill_path(
            &PathBuilder::from_circle(
                (self.size.width.get() as f32) * 0.25,
                (self.size.height.get() as f32) * 0.5,
                100.0,
            )
            .ok_or(eyre!("error creating path"))?,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        pixmap.stroke_path(
            &PathBuilder::from_circle(
                (self.size.width.get() as f32) * 0.75,
                (self.size.height.get() as f32) * 0.5,
                100.0,
            )
            .ok_or(eyre!("error creating path"))?,
            &paint,
            &Stroke {
                width: 5.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );

        buffer
            .present()
            .map_err(|e| eyre!("failed to present softbuffer to window: {e:?}"))?;
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
