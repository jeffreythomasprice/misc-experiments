mod ui;

use bytemuck::Zeroable;
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, Vec2};
use lib::app::{App, Renderer, WindowOptions};
use lib::basic_types::{Affine2, Rect, Vertex2DTextureCoordinateColor};
use lib::colors::Color;
use lib::font::Font;
use lib::fps::FPSCounter;
use lib::math::wrap;
use lib::mesh::Mesh;
use lib::mesh_builder::MeshBuilder;
use lib::pipelines::pipeline2d::{self, Pipeline2d, Transform};
use lib::texture::Texture;
use lib::texture_atlas_font::{
    Alignment, HorizontalAlignment, TextureAtlasFont, VerticalAlignment,
};
use rand::Rng;
use std::sync::Mutex;
use std::time::Duration;
use std::{f32::consts::TAU, sync::Arc};
use tracing::*;
use wgpu::{
    BlendState, Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceConfiguration, TextureView,
};
use winit::dpi::LogicalSize;
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

const SPRITE_SPACE_SIZE: f32 = 1000.0;

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

struct Sprite {
    mesh: Arc<Mesh<Vertex2DTextureCoordinateColor>>,
    texture: Arc<Texture>,
    affine: MovingAffine2,
    transform: pipeline2d::Transform,
}

impl Sprite {
    fn update(&mut self, queue: &Queue, duration: Duration) {
        self.affine.update(duration);
        self.transform
            .enqueue_update(queue, self.affine.affine().into());
    }

    fn render(&self, r: &mut pipeline2d::RenderPass<'_>) {
        r.draw(&self.mesh, &self.texture, &self.transform);
    }
}

struct Demo {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline_no_blend: Pipeline2d,
    pipeline_blend: Pipeline2d,
    sprites: Vec<Sprite>,
    text: ui::Text,
    text_affine: MovingAffine2,
    ortho_window_size: Mat4,
    ortho_sprites: Mat4,
    fps: FPSCounter,
}

impl Demo {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Result<Self> {
        let sprite_texture = Arc::new(Texture::from_image(
            device.clone(),
            queue.clone(),
            Pipeline2d::texture_bindings(),
            &image::load_from_memory_with_format(
                include_bytes!("../assets/rustacean-flat-happy.png"),
                image::ImageFormat::Png,
            )?,
        )?);

        let sprite_texture_size = Vec2::new(
            sprite_texture.width() as f32,
            sprite_texture.height() as f32,
        );
        let sprite_mesh = Arc::new(
            MeshBuilder::<Vertex2DTextureCoordinateColor>::new()
                .rectangle(
                    Rect::from_origin_size(-sprite_texture_size * 0.5, sprite_texture_size),
                    Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    Color::WHITE,
                )
                .create_mesh(&device),
        );

        let mut rng = rand::rng();
        let mut sprites = Vec::new();
        for _ in 0..10 {
            let scale = rng.random_range(0.75..1.25);
            sprites.push(Sprite {
                affine: MovingAffine2 {
                    scale: Vec2::new(scale, scale),
                    angle: rng.random_range(0.0..TAU),
                    translation: Vec2::new(
                        rng.random_range(-(SPRITE_SPACE_SIZE * 0.5)..(SPRITE_SPACE_SIZE * 0.5)),
                        rng.random_range(-(SPRITE_SPACE_SIZE * 0.5)..(SPRITE_SPACE_SIZE * 0.5)),
                    ),
                    angular_velocity: rng
                        .random_range((-45.0f32.to_radians())..=(45.0f32.to_radians())),
                    velocity: Vec2::ZERO,
                },
                mesh: sprite_mesh.clone(),
                texture: sprite_texture.clone(),
                transform: pipeline2d::Transform::new(&device, Mat4::zeroed()),
            });
        }

        let font = Arc::new(Font::new(
            rusttype::Font::try_from_bytes(include_bytes!(
                "../assets/calibri-font-family/calibri-regular.ttf"
            ))
            .ok_or(eyre!("failed to parse font"))?,
        ));
        let texture_atlas_font = Arc::new(Mutex::new(TextureAtlasFont::new(
            device.clone(),
            queue.clone(),
            Pipeline2d::texture_bindings(),
            font.clone(),
            40.0,
        )?));

        let text = ui::Text::new(
            device.clone(),
            queue.clone(),
            texture_atlas_font.clone(),
            glam::Affine2::from_translation(Vec2::new(300.0, 300.0)).into(),
            "".to_owned(),
            Alignment {
                bounds: Rect::from_origin_size(Vec2::new(-150.0, -150.0), Vec2::new(300.0, 300.0)),
                horizontal: HorizontalAlignment::Center,
                vertical: VerticalAlignment::Center,
            },
        );
        let text_affine = MovingAffine2 {
            scale: Vec2::new(1.0, 1.0),
            angle: 0.0,
            translation: Vec2::new(300.0, 300.0),
            angular_velocity: 45.0f32.to_radians(),
            velocity: Vec2::new(0.0, 0.0),
        };

        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline_no_blend: Pipeline2d::new(
                device.clone(),
                queue.clone(),
                surface_configuration,
                BlendState::REPLACE,
            ),
            pipeline_blend: Pipeline2d::new(
                device.clone(),
                queue.clone(),
                surface_configuration,
                BlendState::ALPHA_BLENDING,
            ),
            sprites,
            text,
            text_affine,
            ortho_window_size: Mat4::IDENTITY,
            ortho_sprites: Mat4::IDENTITY,
            fps: FPSCounter::new(),
        })
    }
}

impl Renderer for Demo {
    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        // TODO more helpers for ortho stuff, e.g. exact size, fixed width with aspect ratio, fixed height with aspect ratio, centered on point

        self.ortho_window_size =
            Mat4::orthographic_rh_gl(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);

        let sprite_space_size = LogicalSize::new(
            SPRITE_SPACE_SIZE * (size.width as f32) / (size.height as f32),
            SPRITE_SPACE_SIZE,
        );
        info!(
            "resized to {:?}, sprite space size = {:?}",
            size, sprite_space_size
        );
        self.ortho_sprites = Mat4::orthographic_rh_gl(
            -sprite_space_size.width * 0.5,
            sprite_space_size.width * 0.5,
            sprite_space_size.height * 0.5,
            -sprite_space_size.height * 0.5,
            -1.0,
            1.0,
        );

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
                            load: LoadOp::Clear(Color::CORNFLOWERBLUE.into()),
                            store: StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();

            {
                let mut r = self
                    .pipeline_no_blend
                    .render_pass(&mut render_pass, self.ortho_sprites);

                for sprite in self.sprites.iter() {
                    sprite.render(&mut r);
                }
            }

            {
                let mut r = self
                    .pipeline_blend
                    .render_pass(&mut render_pass, self.ortho_window_size);

                self.text.render(&mut r)?;
            }
        }
        self.queue.submit([encoder.finish()]);

        Ok(())
    }

    fn update(&mut self, duration: Duration) -> Result<()> {
        self.fps.tick(duration);

        for sprite in self.sprites.iter_mut() {
            sprite.update(&self.queue, duration);
        }

        self.text.set_text(format!(
            "FPS: {}\nanother line, yjpqg",
            self.fps.fps_pretty()
        ));
        self.text_affine.update(duration);
        self.text.set_affine(self.text_affine.affine());

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
        .with_env_filter("info,demo=trace,lib=trace")
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut App::new(WindowOptions {
        renderer_factory: Demo::new,
        title: "Experiment".to_string(),
        size: PhysicalSize::new(1024, 768),
        vsync: false,
    }))?;

    Ok(())
}
