mod fps;
mod math;

use std::time::{Duration, SystemTime};

use color_eyre::eyre::Result;
use fps::FPSCounter;
use math::wrap;
use sdl3::{
    event::Event,
    image::LoadTexture,
    iostream::IOStream,
    keyboard::Keycode,
    pixels::Color,
    render::{Canvas, FPoint, FRect, RenderTarget, TextureCreator},
    ttf::Font,
    video::WindowContext,
};

fn main() -> Result<()> {
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();

    color_eyre::install()?;

    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter("info,rust_sdl3=trace")
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

    let texture =
        texture_creator.load_texture_bytes(include_bytes!("../assets/rustacean-flat-happy.png"))?;

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
    let mut rotation = 0.0;

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
        canvas.copy_ex(
            &texture,
            None,
            aligned_rect(
                texture.width() as f32,
                texture.height() as f32,
                FRect::new(0.0, 0.0, width as f32, height as f32),
                HorizontalAlignment::Center,
                VerticalAlignment::Center,
            ),
            rotation,
            FPoint::new(texture.width() as f32 * 0.5, texture.height() as f32 * 0.5),
            false,
            false,
        )?;
        canvas.present();

        if let Some(last_frame_start) = last_frame_start {
            let duration_between_frame_starts = start.duration_since(last_frame_start)?;
            fps.tick(duration_between_frame_starts);

            rotation = wrap(
                rotation + 45.0 * duration_between_frame_starts.as_secs_f64(),
                0.0,
                360.0,
            );
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

fn aligned_rect(
    width: f32,
    height: f32,
    dst: FRect,
    halign: HorizontalAlignment,
    valign: VerticalAlignment,
) -> FRect {
    let x = match halign {
        HorizontalAlignment::Left => dst.x,
        HorizontalAlignment::Center => dst.x + (dst.w - width) * 0.5,
        HorizontalAlignment::Right => dst.x + dst.w - width,
    };
    let y = match valign {
        VerticalAlignment::Top => dst.y,
        VerticalAlignment::Center => dst.y + (dst.h - height) * 0.5,
        VerticalAlignment::Bottom => dst.y + dst.h - height,
    };
    FRect::new(x, y, width, height)
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
    let texture = texture_creator.create_texture_from_surface(font.render(s).blended(color)?)?;
    canvas.copy(
        &texture,
        None,
        aligned_rect(
            texture.width() as f32,
            texture.height() as f32,
            dst,
            halign,
            valign,
        ),
    )?;
    Ok(())
}
