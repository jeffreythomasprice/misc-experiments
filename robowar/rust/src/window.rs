use std::{num::NonZero, process::exit, sync::Arc};

use color_eyre::eyre::{Result, eyre};
use softbuffer::{Context, Surface};
use tracing::*;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

const DESIRED_SIZE: LogicalSize<u32> = LogicalSize::new(1024, 768);

pub trait EventHandler {
    fn render(&mut self, buffer: &mut [u32], width: u32, height: u32) -> Result<()>;
}

struct WindowState<EH: EventHandler> {
    event_handler: EH,
    window: Arc<Window>,
    surface: Surface<Arc<Window>, Arc<Window>>,
}

impl<EH: EventHandler> WindowState<EH> {
    pub fn new(
        event_handler: EH,
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

        let context = Context::new(window.clone())
            .map_err(|e| eyre!("failed to make softbuffer context: {e:?}"))?;
        let surface = Surface::new(&context, window.clone())
            .map_err(|e| eyre!("failed to make softbuffer surface: {e:?}"))?;

        Ok(Self {
            event_handler,
            window,
            surface,
        })
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) -> Result<()> {
        if let (Some(width), Some(height)) = (
            NonZero::new(physical_size.width),
            NonZero::new(physical_size.height),
        ) {
            self.surface
                .resize(width, height)
                .map_err(|e| eyre!("failed to resize softbuffer surface: {e:?}"))?;
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

        self.event_handler.render(&mut buffer, width, height)?;

        buffer
            .present()
            .map_err(|e| eyre!("failed to present softbuffer buffer: {e:?}"))?;
        Ok(())
    }
}

struct App<EH: EventHandler> {
    event_handler: Option<EH>,
    window_state: Option<WindowState<EH>>,
}

impl<EH: EventHandler> App<EH> {
    pub fn new(event_handler: EH) -> Self {
        Self {
            event_handler: Some(event_handler),
            window_state: None,
        }
    }
}

impl<EH: EventHandler> ApplicationHandler for App<EH> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window_state.is_none() {
            if let Some(event_handler) = self.event_handler.take() {
                match WindowState::new(
                    event_handler,
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
            } else {
                error!("no event handler, did we init twice?");
                exit(1);
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
                if let Some(window_state) = &mut self.window_state
                    && let Err(e) = window_state.resize(physical_size)
                {
                    error!("error resizing: {e:?}");
                    exit(1);
                }
            }

            winit::event::WindowEvent::RedrawRequested => {
                if let Some(window_state) = &mut self.window_state
                    && let Err(e) = window_state.render()
                {
                    error!("error rendering: {e:?}");
                    exit(1);
                }
            }

            _ => (),
        }
    }
}

pub fn run<EH: EventHandler>(event_handler: EH) -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut App::new(event_handler))?;

    Ok(())
}
