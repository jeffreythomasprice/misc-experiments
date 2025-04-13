mod app;
mod graphics_utils;
mod misc_utils;
mod wgpu_utils;

use std::cell::RefCell;
use std::time::Duration;
use std::{f32::consts::TAU, sync::Arc};

use app::{App, Renderer};
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, Vec2};
use graphics_utils::basic_types::{Affine2, Color, Rect, Vertex2DTextureCoordinateColor};
use graphics_utils::fps::FPSCounter;
use graphics_utils::mesh_builder::MeshBuilder;
use graphics_utils::pipelines::pipeline_2d_textured::{
    self, Pipeline2DTextured, Pipeline2DTexturedOptions,
};
use graphics_utils::pipelines::{HasDeviceAndQueue, HasTextureBindings};
use graphics_utils::text::Text;
use rand::Rng;
use tracing::*;
use wgpu::{
    BlendState, Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceConfiguration, TextureView,
};
use wgpu_utils::font::Font;
use wgpu_utils::mesh::Mesh;
use wgpu_utils::texture::Texture;
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

/*
TODO more wgpu stuff
https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/



TODO simpler graphics system?

renderer:
- text
- sprites
- instanced sprites, draw the same texture a bunch of times?



TODO texture atlas font
https://docs.rs/binpack2d/latest/binpack2d/
*/

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
        // TODO math helper for wrapping a value to inside range?
        let new_angle =
            (self.angle + self.angular_velocity * duration.as_secs_f32()) % std::f32::consts::TAU;
        self.angle = if new_angle >= 0.0 {
            new_angle
        } else {
            std::f32::consts::TAU - new_angle
        };

        self.translation += self.velocity * duration.as_secs_f32();
    }
}

struct Demo {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline_no_blending: Pipeline2DTextured,
    pipeline_blending: Pipeline2DTextured,
    sprite_mesh: Mesh<Vertex2DTextureCoordinateColor>,
    sprite_texture: Texture,
    sprite_transforms: Vec<MovingAffine2>,
    text: Text,
    ortho: Mat4,
    fps: FPSCounter,
}

impl Demo {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Result<Self> {
        let pipeline_no_blending = Pipeline2DTextured::new(
            device.clone(),
            queue.clone(),
            surface_configuration,
            Pipeline2DTexturedOptions {
                blend_state: BlendState::REPLACE,
            },
        );
        let pipeline_blending = Pipeline2DTextured::new(
            device.clone(),
            queue.clone(),
            surface_configuration,
            Pipeline2DTexturedOptions {
                blend_state: BlendState::ALPHA_BLENDING,
            },
        );

        let sprite_texture = Texture::from_image(
            device.clone(),
            queue.clone(),
            Pipeline2DTextured::texture_binding(),
            image::load_from_memory_with_format(
                include_bytes!("../assets/rustacean-flat-happy.png"),
                image::ImageFormat::Png,
            )?,
        )?;
        info!("texture size: {:?}", sprite_texture.size());

        let sprite_mesh = MeshBuilder::<Vertex2DTextureCoordinateColor>::new()
            .rectangle(
                Rect::from_origin_size(
                    Vec2::new(
                        -(sprite_texture.width() as f32 / 2.0),
                        -(sprite_texture.height() as f32 / 2.0),
                    ),
                    Vec2::new(
                        sprite_texture.width() as f32,
                        sprite_texture.height() as f32,
                    ),
                ),
                Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            )
            .create_mesh(&device);

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
        let text = Text::new(
            device.clone(),
            queue.clone(),
            Pipeline2DTextured::texture_binding(),
            font.clone(),
            40.0,
        );

        Ok(Self {
            device,
            queue,
            pipeline_no_blending,
            pipeline_blending,
            sprite_mesh,
            sprite_texture,
            sprite_transforms,
            text,
            ortho: Mat4::IDENTITY,
            fps: FPSCounter::new(),
        })
    }
}

impl Renderer for Demo {
    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        self.ortho =
            Mat4::orthographic_rh_gl(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);

        Ok(())
    }

    fn render(&mut self, texture_view: TextureView) -> Result<()> {
        self.text
            .update(&format!("FPS: {}", self.fps.fps_pretty()))?;

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
            });
            let render_pass = self
                .pipeline_no_blending
                .render(render_pass, self.ortho, |r| {
                    for transform in self.sprite_transforms.iter() {
                        r.render(
                            &self.sprite_texture,
                            transform.affine().into(),
                            &self.sprite_mesh,
                        )?;
                    }
                    Ok(())
                })?;
            self.pipeline_blending
                .render(render_pass, self.ortho, |r| {
                    let (mesh, texture) = self.text.get().ok_or(eyre!("failed to render text"))?;
                    let transform: Affine2 =
                        glam::Affine2::from_translation(Vec2::new(100.0, 50.0)).into();
                    r.render(texture, transform.into(), mesh)?;
                    Ok(())
                })?;
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
