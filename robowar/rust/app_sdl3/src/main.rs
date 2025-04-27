mod renderer;

use std::time::{Duration, SystemTime};

use bytemuck::Zeroable;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use lib::{
    fps::FPSCounter,
    geom::{Affine2f32, Rectf32, Vec2f32, Vec2u32},
    graphics::{
        camera::Camera,
        color::{RGBAf32, RGBAu8},
        renderer::{Material, Renderer, Texture},
    },
};
use sdl3::{
    event::{Event, WindowEvent},
    iostream::IOStream,
    keyboard::Keycode,
    pixels::{Color, PixelFormat, PixelMasks},
    render::{Canvas, FRect, RenderTarget, TextureAccess, TextureCreator},
    surface,
    sys::{pixels::SDL_PixelFormat, video},
    ttf::Font,
    video::WindowContext,
};

fn main() -> Result<()> {
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

    let mut renderer = renderer::Renderer::new(window.into_canvas());

    let font = sdl_ttf_context.load_font_from_iostream(
        IOStream::from_bytes(include_bytes!(
            "../../assets/calibri-font-family/calibri-regular.ttf"
        ))?,
        40.0,
    )?;

    let test_texture = {
        let width = 256;
        let height = 256;
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            let b = (y as f64 / (height - 1) as f64 * 255.0) as u8;
            for x in 0..width {
                let a = (x as f64 / (width - 1) as f64 * 255.0) as u8;
                pixels.push(RGBAu8::new(a, b, a, 255));
            }
        }
        renderer.create_texture_pixels(Vec2u32::new(width as u32, height as u32), &mut pixels)?
    };

    let mut camera = {
        let (width, height, _) = renderer.canvas().logical_size();
        Camera::new(
            Rectf32::with_origin_size(Vec2f32::zeroed(), Vec2f32::new(256.0, 256.0)),
            Vec2f32::new(16.0, 16.0),
            // TODO get window size from sdl
            Vec2f32::new(width as f32, height as f32),
        )?
    };

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

                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event: WindowEvent::Resized(_, _),
                } => {
                    let (width, height, _) = renderer.canvas().logical_size();
                    camera.set_window_size(Vec2f32::new(width as f32, height as f32));
                }

                // TODO event handler for keys and mouse that move camera around
                _ => (),
            }
        }

        let (width, height, _) = renderer.canvas().logical_size();
        renderer.clear_screen(RGBAf32::CORNFLOWERBLUE)?;
        renderer.set_ortho(Rectf32::with_origin_size(
            Vec2f32::zeroed(),
            Vec2f32::new(width as f32, height as f32),
        ))?;
        // TODO re-implement using Renderer
        // draw_string(
        //     &mut canvas,
        //     &texture_creator,
        //     &font,
        //     &format!("FPS: {}", fps.fps_pretty()),
        //     Color::RGB(255, 255, 255),
        //     FRect::new(0.0, 0.0, width as f32, height as f32),
        //     HorizontalAlignment::Left,
        //     VerticalAlignment::Top,
        // )?;
        // TODO update ortho based on camera
        renderer.fill_rect(
            Rectf32::with_origin_size(
                Vec2f32::zeroed(),
                Vec2f32::new(test_texture.width() as f32, test_texture.height() as f32),
            ),
            Affine2f32::identity(),
            &mut Material {
                color: None,
                texture: todo!(),
                blend: false,
            },
        )?;
        // canvas.copy(
        //     &test_texture,
        //     None,
        //     Some(FRect::new(
        //         0.0,
        //         0.0,
        //         test_texture.width() as f32,
        //         test_texture.height() as f32,
        //     )),
        // )?;
        renderer.present()?;

        if let Some(last_frame_start) = last_frame_start {
            let duration_between_frame_starts = start.duration_since(last_frame_start)?;
            fps.tick(duration_between_frame_starts);

            // TODO do updates here
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
