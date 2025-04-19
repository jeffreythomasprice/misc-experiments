mod fps;

use std::time::{Duration, SystemTime};

use color_eyre::eyre::{Result, eyre};
use fps::FPSCounter;
use sdl3::{
    event::Event,
    iostream::IOStream,
    keyboard::Keycode,
    pixels::Color,
    rect::Point,
    render::{Canvas, FRect, RenderTarget, TextureCreator},
    ttf::Font,
    video::WindowContext,
};

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
    let sdl_ttf_context = sdl3::ttf::init()?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Experiment", 1024, 768)
        .position_centered()
        .resizable()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas();
    canvas.set_logical_size(
        640,
        480,
        sdl3::sys::render::SDL_RendererLogicalPresentation::LETTERBOX,
    )?;
    let texture_creator = canvas.texture_creator();

    let font = sdl_ttf_context.load_font_from_iostream(
        IOStream::from_bytes(include_bytes!(
            "../assets/calibri-font-family/calibri-regular.ttf"
        ))?,
        24.0,
    )?;

    let mut event_pump = sdl_context.event_pump()?;

    const DESIRED_FPS: u32 = 60;
    let desired_duration_per_frame = Duration::from_secs_f64(1.0 / (DESIRED_FPS as f64));
    let mut fps = FPSCounter::new();
    let mut last_frame_start = None;

    'running: loop {
        let start = SystemTime::now();

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
        // for y in 0..height {
        //     let b = ((y as f64) / (height as f64) * 255.0) as u8;
        //     for x in 0..width {
        //         let a = ((x as f64) / (width as f64) * 255.0) as u8;
        //         canvas.set_draw_color(Color::RGB(a, b, a));
        //         canvas.draw_point(Point::new(x as i32, y as i32))?;
        //     }
        // }
        canvas.set_draw_color(Color::RGB(64, 64, 64));
        canvas.clear();
        draw_string(
            &mut canvas,
            &texture_creator,
            &font,
            &format!("FPS: {}", fps.fps_pretty()),
            Color::RGB(255, 255, 255),
            FRect::new(0.0, 0.0, width as f32, height as f32),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
        )?;
        canvas.present();

        if let Some(last_frame_start) = last_frame_start {
            let duration_between_frame_starts = start.duration_since(last_frame_start)?;
            fps.tick(duration_between_frame_starts);
        }
        last_frame_start = Some(start);

        let end = SystemTime::now();
        let duration_this_frame = end.duration_since(start)?;
        if desired_duration_per_frame > duration_this_frame {
            ::std::thread::sleep(desired_duration_per_frame - duration_this_frame);
        }
    }

    Ok(())
}

enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

fn draw_string<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    s: &str,
    color: Color,
    dst: FRect,
    halign: HorizontalAlignment,
    valign: VerticalAlignment,
) -> Result<()> {
    let surface = font.render(s).blended(color)?;
    let texture = texture_creator.create_texture_from_surface(surface)?;
    let x = match halign {
        HorizontalAlignment::Left => dst.x,
        HorizontalAlignment::Center => dst.x + (dst.w - texture.width() as f32) * 0.5,
        HorizontalAlignment::Right => dst.x + dst.w - texture.width() as f32,
    };
    let y = match valign {
        VerticalAlignment::Top => dst.y,
        VerticalAlignment::Center => dst.y + (dst.h - texture.height() as f32) * 0.5,
        VerticalAlignment::Bottom => dst.y + dst.h - texture.height() as f32,
    };
    canvas.copy(
        &texture,
        None,
        Some(FRect::new(
            x,
            y,
            texture.width() as f32,
            texture.height() as f32,
        )),
    )?;
    Ok(())
}
