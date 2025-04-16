mod app;
mod graphics_utils;
mod misc_utils;
mod wgpu_utils;

use std::time::Duration;
use std::{f32::consts::TAU, sync::Arc};

use app::{App, Renderer};
use bytemuck::Zeroable;
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, Vec2};
use graphics_utils::basic_types::{Affine2, Rect};
use graphics_utils::colors::Color;
use graphics_utils::font::Font;
use graphics_utils::fps::FPSCounter;
use graphics_utils::simple_renderer::SimpleRenderer;
use graphics_utils::texture_atlas_font::TextureAtlasFont;
use misc_utils::math::wrap;
use rand::Rng;
use tracing::*;
use wgpu::{
    Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    SurfaceConfiguration, TextureView,
};
use wgpu_utils::texture::Texture;
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

struct MovingAffine2 {
    scale: Vec2,
    angle: f32,
    translation: Vec2,

    angular_velocity: f32,
    velocity: Vec2,
}

impl MovingAffine2 {
    pub fn affine(&self) -> Affine2 {
        glam::Affine2::from_scale_angle_translation(self.scale, self.angle, self.translation).into()
    }

    pub fn update(&mut self, duration: Duration) {
        self.angle = wrap(
            self.angle + self.angular_velocity * duration.as_secs_f32(),
            0.0,
            std::f32::consts::TAU,
        );
        self.translation += self.velocity * duration.as_secs_f32();
    }
}

struct Demo {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer: SimpleRenderer,
    sprite_texture: Texture,
    sprite_transforms: Vec<MovingAffine2>,
    texture_atlas_font: TextureAtlasFont,
    ortho: Mat4,
    fps: FPSCounter,
}

impl Demo {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Result<Self> {
        let renderer = SimpleRenderer::new(device.clone(), queue.clone(), surface_configuration);

        let sprite_texture = Texture::from_image(
            device.clone(),
            queue.clone(),
            SimpleRenderer::texture_bindings(),
            &image::load_from_memory_with_format(
                include_bytes!("../assets/rustacean-flat-happy.png"),
                image::ImageFormat::Png,
            )?,
        )?;
        info!("texture size: {:?}", sprite_texture.size());

        let mut rng = rand::rng();
        let mut sprite_transforms = Vec::new();
        for _ in 0..10 {
            let scale = rng.random_range(0.75..1.25);
            sprite_transforms.push(MovingAffine2 {
                scale: Vec2::new(scale, scale),
                angle: rng.random_range(0.0..TAU),
                translation: Vec2::new(rng.random_range(0.0..500.0), rng.random_range(0.0..500.0)),
                angular_velocity: rng
                    .random_range((-45.0f32.to_radians())..=(45.0f32.to_radians())),
                velocity: Vec2::ZERO,
            });
        }

        let font = Arc::new(Font::new(
            rusttype::Font::try_from_bytes(include_bytes!(
                "../assets/calibri-font-family/calibri-regular.ttf"
            ))
            .ok_or(eyre!("failed to parse font"))?,
        ));
        let texture_atlas_font = TextureAtlasFont::new(
            device.clone(),
            queue.clone(),
            SimpleRenderer::texture_bindings(),
            font.clone(),
            40.0,
        )?;

        Ok(Self {
            device,
            queue,
            renderer,
            sprite_texture,
            sprite_transforms,
            texture_atlas_font,
            ortho: Mat4::IDENTITY,
            fps: FPSCounter::new(),
        })
    }
}

impl Renderer for Demo {
    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        self.ortho =
            Mat4::orthographic_rh_gl(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);
        self.renderer.set_viewport(Rect::from_origin_size(
            Vec2::zeroed(),
            Vec2::new(size.width as f32, size.height as f32),
        ));

        Ok(())
    }

    fn render(&mut self, texture_view: TextureView) -> Result<()> {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder
                .begin_render_pass(&RenderPassDescriptor {
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(wgpu::Color {
                                r: 0.25,
                                g: 0.25,
                                b: 0.25,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();

            let mut r = self.renderer.render_pass(&mut render_pass);

            r.set_blend(false);
            for transform in self.sprite_transforms.iter() {
                let size = Vec2::new(
                    self.sprite_texture.width() as f32,
                    self.sprite_texture.height() as f32,
                );
                r.fill_rect_texture(
                    transform.affine(),
                    Rect::from_origin_size(-size * 0.5, size),
                    &self.sprite_texture,
                    Rect::from_origin_size(Vec2::zeroed(), Vec2::new(1.0, 1.0)),
                    Color::WHITE,
                )?;
            }

            r.set_blend(true);
            // TODO should allow drawing inside a rect, with alignmnet
            r.draw_textured_string(
                &mut self.texture_atlas_font,
                &format!("FPS: {}\nanother line, yjpqg", self.fps.fps_pretty()),
                glam::Affine2::from_translation(Vec2::new(100.0, 50.0)).into(),
                Color::WHITE,
            )?;
        }
        self.queue.submit([encoder.finish()]);

        Ok(())
    }

    fn update(&mut self, duration: Duration) -> Result<()> {
        self.fps.tick(duration);

        for transform in self.sprite_transforms.iter_mut() {
            transform.update(duration);
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();

    color_eyre::install()?;

    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter("info,game=trace")
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut App::new(Demo::new))?;

    Ok(())
}
