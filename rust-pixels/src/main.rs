use anyhow::Result;
use env_logger::DEFAULT_FILTER_ENV;
use log::*;
use pixels::{Pixels, SurfaceTexture};
use std::env;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

struct AppWindow {
    window: Window,
    pixels: Pixels,
}

struct App {
    close_requested: bool,
    window: Option<AppWindow>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Experiment"))
            .expect("failed to create window");
        let width = window.inner_size().width;
        let height = window.inner_size().height;
        let surface_texture = SurfaceTexture::new(width, height, &window);
        let pixels = Pixels::new(width, height, surface_texture).expect("failed to create pixels");
        self.window = Some(AppWindow { window, pixels });
    }

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_mut() {
                    window.window.pre_present_notify();
                    draw(
                        window.window.inner_size().width,
                        window.window.inner_size().height,
                        window.pixels.frame_mut(),
                    );
                    if let Err(e) = window.pixels.render() {
                        error!("pixels render error: {e:?}");
                        self.close_requested = true;
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(window) = self.window.as_mut() {
                    if let Err(e) = window.pixels.resize_surface(size.width, size.height) {
                        error!("pixels resize error: {e:?}");
                        self.close_requested = true;
                    }
                    // TODO should be able to use a fixed size buffer and let it do the scaling to fit the window
                    if let Err(e) = window.pixels.resize_buffer(size.width, size.height) {
                        error!("pixels resize error: {e:?}");
                        self.close_requested = true;
                    }
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
            window.window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

fn main() -> Result<()> {
    if let Err(_) = env::var(DEFAULT_FILTER_ENV) {
        env::set_var(DEFAULT_FILTER_ENV, "info, rust_pixels=trace");
    }
    env_logger::builder().parse_default_env().init();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop
        .run_app(&mut App {
            close_requested: false,
            window: None,
        })
        .expect("event loop failed");

    Ok(())
}

fn draw(width: u32, height: u32, pixels: &mut [u8]) {
    for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
        let x = i % (width as usize);
        let y = i / (width as usize);

        let a = ((x as f64) / (width as f64) * 255.0) as u8;
        let b = ((y as f64) / (height as f64) * 255.0) as u8;

        pixel.copy_from_slice(&[a, b, 255 - a, 0xff]);
    }
}
