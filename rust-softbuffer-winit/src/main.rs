use std::num::NonZero;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use log::Level;
use log::*;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

fn main() -> Result<()> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::new())?;

    Ok(())
}

struct AppWindow {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

struct App {
    window: Option<AppWindow>,
    close_requested: bool,
    size: Option<PhysicalSize<NonZero<u32>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            close_requested: false,
            size: None,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        info!("TODO resize: {size:?}");

        self.size = match (NonZero::new(size.width), NonZero::new(size.height)) {
            (Some(width), Some(height)) => Some(PhysicalSize { width, height }),
            _ => None,
        };

        if let Some(PhysicalSize { width, height }) = self.size {
            if let Some(AppWindow { surface, .. }) = &mut self.window {
                if let Err(e) = surface.resize(width, height) {
                    error!("error resizing: {e:?}");
                    self.close_requested = true;
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = match || -> Result<AppWindow> {
            let attributes = Window::default_attributes();
            #[cfg(target_arch = "wasm32")]
            let attributes =
                winit::platform::web::WindowAttributesExtWebSys::with_append(attributes, true);
            info!("using window attributes: {attributes:?}");
            let window = Rc::new(event_loop.create_window(attributes)?);
            let context = Context::new(window.clone())
                .map_err(|e| anyhow!("failed to create context: {e:?}"))?;
            let surface = Surface::new(&context, window.clone())
                .map_err(|e| anyhow!("failed to create surface: {e:?}"))?;
            Ok(AppWindow {
                window: window.clone(),
                surface,
            })
        }() {
            Ok(window) => {
                window.window.request_redraw();
                Some(window)
            }
            Err(e) => {
                error!("failed to make window: {e:?}");
                self.close_requested = true;
                None
            }
        };
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(PhysicalSize { width, height }) = self.size {
                    if let Some(AppWindow { surface, .. }) = &mut self.window {
                        if let Err(e) = move || -> Result<()> {
                            let mut buf = surface
                                .buffer_mut()
                                .map_err(|e| anyhow!("error getting buffer: {e:?}"))?;
                            let width = width.get() as usize;
                            let height = height.get() as usize;
                            for (i, ptr) in buf.iter_mut().enumerate() {
                                let y = i / width;
                                let x = i % width;
                                let a = ((x as f64) / (width as f64) * 255.) as u8;
                                let b = ((y as f64) / (height as f64) * 255.) as u8;
                                let red = a as u32;
                                let green = b as u32;
                                let blue = a as u32;
                                // TODO color based on coords
                                *ptr = (red << 16) | (green << 8) | blue;
                            }
                            buf.present().map_err(|e| {
                                anyhow!("failed to present new buffer to surface: {e:?}")
                            })?;
                            Ok(())
                        }() {
                            error!("error rendering: {e:?}");
                            self.close_requested = true;
                        }
                    }
                }
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
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

        // todo!()
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.close_requested {
            event_loop.exit();
            return;
        }

        if let Some(window) = &self.window {
            window.window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
