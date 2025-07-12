mod math;
mod parser;
mod simulation;
mod window;

use color_eyre::eyre::{self, Result, eyre};
use tiny_skia::{FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::window::{EventHandler, run};

struct Demo {}

impl EventHandler for Demo {
    fn render(&mut self, buffer: &mut [u32], width: u32, height: u32) -> Result<()> {
        let pixels: &mut [u8] = bytemuck::try_cast_slice_mut(buffer)
            .map_err(|e| eyre!("failed to cast buffer to u8 slice: {e:?}"))?;

        for index in 0..(width * height) {
            let x = index % width;
            let y = index / width;
            let a = ((x as f64) / (width as f64) * 255.) as u8;
            let b = ((y as f64) / (height as f64) * 255.) as u8;
            let r = a;
            let g = b;
            let b = a;
            let i = (index * 4) as usize;
            pixels[i + 0] = b as u8;
            pixels[i + 1] = g as u8;
            pixels[i + 2] = r as u8;
            pixels[i + 3] = 255;
        }

        let mut pixmap = PixmapMut::from_bytes(pixels, width, height)
            .ok_or(eyre!("error creating skia pixmap"))?;
        let mut paint = Paint::default();
        // tiny_skia assumes colors are bgra
        // softbuffer assumes colors are rgba
        paint.set_color_rgba8(0, 0, 255, 255);
        paint.anti_alias = true;
        pixmap.fill_path(
            &PathBuilder::from_circle((width as f32) * 0.25, (height as f32) * 0.5, 100.0)
                .ok_or(eyre!("error creating path"))?,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        pixmap.stroke_path(
            &PathBuilder::from_circle((width as f32) * 0.75, (height as f32) * 0.5, 100.0)
                .ok_or(eyre!("error creating path"))?,
            &paint,
            &Stroke {
                width: 5.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );

        Ok(())
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

    run(Demo {})
}
