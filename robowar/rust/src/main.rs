mod math;
mod parser;
mod simulation;
mod window;

use std::{rc::Rc, time::Duration};

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
    offset: Vec2<f64>,
}

impl Camera {
    pub fn new(source_bounds: Rect<f64>, destination_bounds: Rect<f64>) -> Self {
        let scale = (destination_bounds.width() / source_bounds.width())
            .min(destination_bounds.height() / source_bounds.height());
        let offset_x = (destination_bounds.width() - source_bounds.width() * scale) * 0.5;
        let offset_y = (destination_bounds.height() - source_bounds.height() * scale) * 0.5;
        Self {
            source_bounds,
            destination_bounds,
            scale,
            offset: Vec2::new(offset_x, offset_y),
        }
    }

    pub fn tinyskia_transform(&self) -> Transform {
        Transform::from_scale(self.scale as f32, self.scale as f32)
            .pre_translate(
                -self.source_bounds.minimum().x as f32,
                -self.source_bounds.minimum().y as f32,
            )
            .post_translate(
                self.destination_bounds.minimum().x as f32,
                self.destination_bounds.minimum().y as f32,
            )
            .post_translate(self.offset.x as f32, self.offset.y as f32)
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
        let border = 50.;
        let camera = Camera::new(
            self.simulation
                .physics_environment()
                .bounding_box()?
                .clone(),
            // Rect::new_with_origin_size(Vec2::new(0.0, 0.0), Vec2::new(width as f64, height as f64)),
            Rect::new_with_points(&[
                Vec2::new(border, border),
                Vec2::new(width as f64 - border, height as f64 - border),
            ])
            .unwrap(),
        );

        let pixels_abgr: &mut [u8] = bytemuck::try_cast_slice_mut(buffer)
            .map_err(|e| eyre!("failed to cast buffer to u8 slice: {e:?}"))?;

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
        for line in self.simulation.physics_environment().get_line_segments()? {
            let a = *line.origin();
            let b = *line.origin() + *line.delta();
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
            camera.tinyskia_transform(),
            None,
        );

        for actor in self.simulation.physics_environment().get_actors() {
            let actor = actor.borrow();
            let position = actor.position()?;
            let radius = actor.radius();
            let circle =
                PathBuilder::from_circle(position.x as f32, position.y as f32, radius as f32)
                    .ok_or(eyre!("error creating circle path"))?;
            paint.set_color_rgba8(255, 255, 255, 255);
            pixmap.fill_path(
                &circle,
                &paint,
                FillRule::Winding,
                camera.tinyskia_transform(),
                None,
            );
            paint.set_color_rgba8(0, 0, 0, 255);
            pixmap.stroke_path(
                &circle,
                &paint,
                &Stroke {
                    width: 2.0,
                    ..Default::default()
                },
                camera.tinyskia_transform(),
                None,
            );

            let turret_end = position + actor.turret_angle().cos_sin_vec2() * radius;
            let mut turret = PathBuilder::new();
            turret.move_to(position.x as f32, position.y as f32);
            turret.line_to(turret_end.x as f32, turret_end.y as f32);
            paint.set_color_rgba8(255, 0, 0, 255);
            pixmap.stroke_path(
                &turret
                    .finish()
                    .ok_or(eyre!("error finishing turret path"))?,
                &paint,
                &Stroke {
                    width: 1.0,
                    ..Default::default()
                },
                camera.tinyskia_transform(),
                None,
            );
        }

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

    fn update(&mut self, elapsed_time: Duration) -> Result<()> {
        self.simulation.update(elapsed_time);
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

    let program = Rc::new(
        parser::parse(
            r"
                set velocity_x, 250
                set velocity_y, 200

                loop:
                    add velocity_y, velocity_y, 10
                    jmp loop
                ",
        )
        .map_err(|e| eyre!("{e:?}"))?
        .runnable_program,
    );
    let mut robots = Vec::new();
    for _ in 0..10 {
        robots.push(program.clone());
    }

    run(Demo::new(simulation::simulation::Simulation::new(
        physics::Environment::new_standard_rectangle(Rect::new_with_origin_size(
            Vec2::new(0.0, 0.0),
            Vec2::new(500.0, 500.0),
        )),
        robots,
        (10.0)..=(20.0),
    )?))
}
