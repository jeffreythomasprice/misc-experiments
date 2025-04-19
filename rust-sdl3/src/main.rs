use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use sdl3::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();

    color_eyre::install()?;

    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter("info,game=trace")
        .init();

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Experiment", 1024, 768)
        .position_centered()
        .resizable()
        .build()?;

    let mut canvas = window.into_canvas();
    canvas.set_logical_size(
        1024,
        768,
        sdl3::sys::render::SDL_RendererLogicalPresentation::LETTERBOX,
    )?;

    let mut event_pump = sdl_context.event_pump()?;

    const DESIRED_FPS: u32 = 30;
    let sleep_time = Duration::from_secs_f64(1.0 / (DESIRED_FPS as f64));
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => (),
            }
        }

        let (width, height, _) = canvas.logical_size();
        for y in 0..height {
            let b = ((y as f64) / (height as f64) * 255.0) as u8;
            for x in 0..width {
                let a = ((x as f64) / (width as f64) * 255.0) as u8;
                canvas.set_draw_color(Color::RGB(a, b, a));
                canvas.draw_point(Point::new(x as i32, y as i32))?;
            }
        }
        canvas.present();

        ::std::thread::sleep(sleep_time);
    }

    Ok(())
}
