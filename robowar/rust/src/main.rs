mod math;
mod parser;
mod simulation;
mod window;

use color_eyre::eyre::{self, Result, eyre};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    math::{Rect, Vec2},
    simulation::physics,
    window::{EventHandler, run},
};

// TODO move me
struct Camera {
    source_bounds: Rect<f64>,
    destination_bounds: Rect<f64>,
    scale: f64,
}

impl Camera {
    pub fn new(source_bounds: Rect<f64>, destination_bounds: Rect<f64>) -> Self {
        Self {
            source_bounds,
            destination_bounds,
            scale: (destination_bounds.width() / source_bounds.width())
                .min(destination_bounds.height() / source_bounds.height()),
        }
    }

    pub fn transform(&self, point: Vec2<f64>) -> Vec2<f64> {
        // TODO should be centered? movable camera with an offset that can move around inside the scaled area?
        (point - self.source_bounds.minimum()) * self.scale + self.destination_bounds.minimum()
    }

    pub fn scale_scalar(&self, value: f64) -> f64 {
        value * self.scale
    }

    pub fn scale_vector(&self, vector: Vec2<f64>) -> Vec2<f64> {
        vector * self.scale
    }
}

struct Demo {
    simulation: simulation::simulation::Simulation,
    pixels_rgba: Vec<u8>,
}

impl Demo {
    fn new(simulation: simulation::simulation::Simulation) -> Self {
        Self {
            simulation,
            pixels_rgba: Vec::new(),
        }
    }
}

impl EventHandler for Demo {
    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.pixels_rgba.resize((width * height * 4) as usize, 0);
        Ok(())
    }

    fn render(&mut self, buffer: &mut [u32], width: u32, height: u32) -> Result<()> {
        let camera = Camera::new(
            self.simulation.physics_environment().bounding_box().clone(),
            Rect::new_with_origin_size(Vec2::new(0.0, 0.0), Vec2::new(width as f64, height as f64)),
        );

        let pixels_abgr: &mut [u8] = bytemuck::try_cast_slice_mut(buffer)
            .map_err(|e| eyre!("failed to cast buffer to u8 slice: {e:?}"))?;

        // TODO not needed?
        // for index in 0..(width * height) {
        //     let x = index % width;
        //     let y = index / width;
        //     let a = ((x as f64) / (width as f64) * 255.) as u8;
        //     let b = ((y as f64) / (height as f64) * 255.) as u8;
        //     let r = a;
        //     let g = b;
        //     let b = a;
        //     let i = (index * 4) as usize;
        //     self.pixels_rgba[i + 0] = b as u8;
        //     self.pixels_rgba[i + 1] = g as u8;
        //     self.pixels_rgba[i + 2] = r as u8;
        //     self.pixels_rgba[i + 3] = 255;
        // }

        let mut pixmap = PixmapMut::from_bytes(&mut self.pixels_rgba, width, height)
            .ok_or(eyre!("error creating skia pixmap"))?;
        let mut paint = Paint::default();

        paint.anti_alias = true;

        paint.set_color_rgba8(64, 128, 255, 255);
        pixmap.fill_rect(
            tiny_skia::Rect::from_xywh(0., 0., width as f32, height as f32).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );

        paint.set_color_rgba8(0, 0, 0, 255);
        let mut path = PathBuilder::new();
        for line in self.simulation.physics_environment().get_line_segments() {
            let a = *line.origin();
            let b = *line.origin() + *line.delta();
            let a = camera.transform(a);
            let b = camera.transform(b);
            tracing::info!("TODO a->b = {a:?} -> {b:?}");
            path.move_to(a.x as f32, a.y as f32);
            path.line_to(b.x as f32, b.y as f32);
        }
        pixmap.stroke_path(
            &path.finish().ok_or(eyre!("error finishing path"))?,
            &paint,
            &Stroke {
                width: 1.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );

        paint.set_color_rgba8(255, 0, 0, 255);
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

        for i in (0..self.pixels_rgba.len()).step_by(4) {
            let r = self.pixels_rgba[i + 0];
            let g = self.pixels_rgba[i + 1];
            let b = self.pixels_rgba[i + 2];
            let a = self.pixels_rgba[i + 3];
            pixels_abgr[i + 0] = b;
            pixels_abgr[i + 1] = g;
            pixels_abgr[i + 2] = r;
            pixels_abgr[i + 3] = a;
        }

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

    run(Demo::new(simulation::simulation::Simulation::new(
        physics::Environment::new_standard_rectangle(Rect::new_with_origin_size(
            Vec2::new(0.0, 0.0),
            Vec2::new(500.0, 500.0),
        )),
        vec![],
        (1.0)..=(5.0),
    )))
}
