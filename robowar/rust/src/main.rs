mod math;
mod parser;
mod simulation;

use std::{num::NonZero, process::exit, sync::Arc};

use color_eyre::eyre::{Result, eyre};
use softbuffer::{Context, Surface};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

const DESIRED_SIZE: LogicalSize<u32> = LogicalSize::new(1024, 768);

struct WindowState {
    window: Arc<Window>,
    surface: Surface<Arc<Window>, Arc<Window>>,
}

struct App {
    window_state: Option<WindowState>,
}

impl WindowState {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        size: LogicalSize<u32>,
    ) -> Result<Self> {
        let window_size = PhysicalSize::new(size.width, size.height);
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

        // TODO softbuffer context initialization would go here
        let context = Context::new(window.clone())
            .map_err(|e| eyre!("failed to make softbuffer context: {e:?}"))?;
        let surface = Surface::new(&context, window.clone())
            .map_err(|e| eyre!("failed to make softbuffer surface: {e:?}"))?;

        Ok(Self { window, surface })
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) -> Result<()> {
        match (
            NonZero::new(physical_size.width),
            NonZero::new(physical_size.height),
        ) {
            (Some(width), Some(height)) => {
                self.surface
                    .resize(width, height)
                    .map_err(|e| eyre!("failed to resize softbuffer surface: {e:?}"))?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| eyre!("failed to get softbuffer buffer: {e:?}"))?;

        let (width, height) = {
            let size = self.window.inner_size();
            (size.width, size.height)
        };
        for index in 0..(width * height) {
            let x = index % width;
            let y = index / width;
            let r = x % 255;
            let g = y % 255;
            let b = 255 - (x % 255);
            buffer[index as usize] = b | (g << 8) | (r << 16);
        }

        buffer
            .present()
            .map_err(|e| eyre!("failed to present softbuffer buffer: {e:?}"))?;
        Ok(())
    }
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

fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut App::new())?;

    Ok(())
}
