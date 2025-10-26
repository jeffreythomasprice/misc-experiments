use std::{
    collections::HashMap,
    thread,
    time::{Duration, Instant},
};
use tracing::*;

use color_eyre::eyre::{Result, eyre};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    sys::video::{
        SDL_GL_SetSwapInterval, SDL_SetWindowSurfaceVSync, SDL_WINDOW_SURFACE_VSYNC_DISABLED,
    },
    video::Window,
};

pub struct KeyboardState {
    state: HashMap<Keycode, bool>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub fn is_pressed(&self, keycode: Keycode) -> bool {
        *self.state.get(&keycode).unwrap_or(&false)
    }

    pub fn set_pressed(&mut self, keycode: Keycode, pressed: bool) {
        self.state.insert(keycode, pressed);
    }
}

pub struct AppState {
    pub window: Window,
    pub keyboard: KeyboardState,
}

pub trait App {
    fn render(&self, state: &AppState) -> Result<()>;
    fn update(&mut self, state: &AppState, elapsed_time: Duration) -> Result<()>;
    fn event(&mut self, state: &AppState, event: &Event) -> Result<()>;
}

pub fn sdl_main<A, F>(app_factory: F) -> Result<()>
where
    A: App,
    F: FnOnce() -> Result<A>,
{
    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    let sdl_context = sdl3::init()?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Experiment", 1024, 768)
        .position_centered()
        .opengl()
        .build()?;

    let gl_context = window.gl_create_context()?;
    window.gl_make_current(&gl_context)?;

    // turn off vsync
    unsafe {
        SDL_SetWindowSurfaceVSync(window.raw(), SDL_WINDOW_SURFACE_VSYNC_DISABLED);
        SDL_GL_SetSwapInterval(0);
    }

    gl::load_with(|s| {
        video_subsystem
            .gl_get_proc_address(s)
            .ok_or(eyre!("failed to get GL proc address: {}", s))
            .unwrap() as *const _
    });

    let mut app = app_factory()?;

    let mut app_state = AppState {
        window,
        keyboard: KeyboardState::new(),
    };

    const DESIRED_FPS: f64 = 60.0;
    const DESIRED_FRAME_DURATION: Duration = Duration::from_nanos(
        ((Duration::from_secs(1).as_nanos() as f64) / DESIRED_FPS).ceil() as u64,
    );
    let mut last_tick: Option<Instant> = None;

    'mainLoop: loop {
        let mut event_pump = sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainLoop,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    app_state.keyboard.set_pressed(keycode, true);
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    app_state.keyboard.set_pressed(keycode, false);
                }

                _ => (),
            };

            app.event(&app_state, &event)?;
        }

        app.render(&app_state)?;
        app_state.window.gl_swap_window();

        let now = Instant::now();
        if let Some(previous) = last_tick {
            let elapsed_time = now - previous;
            app.update(&app_state, elapsed_time)?;
            if elapsed_time >= DESIRED_FRAME_DURATION {
                thread::yield_now();
            } else {
                thread::sleep(DESIRED_FRAME_DURATION - elapsed_time);
            }
        }
        last_tick = Some(now);
    }

    Ok(())
}
