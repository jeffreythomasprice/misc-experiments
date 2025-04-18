mod app;
mod graphics_utils;
mod misc_utils;
mod renderers;
mod wgpu_utils;

use std::sync::Mutex;
use std::time::Duration;
use std::{f32::consts::TAU, sync::Arc};

use app::{App, Renderer};
use bytemuck::Zeroable;
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, Vec2};
use graphics_utils::basic_types::{Affine2, Rect, Vertex2DTextureCoordinateColor};
use graphics_utils::colors::Color;
use graphics_utils::font::Font;
use graphics_utils::fps::FPSCounter;
use graphics_utils::mesh_builder::MeshBuilder;
use graphics_utils::texture_atlas_font::TextureAtlasFont;
use misc_utils::math::wrap;
use rand::Rng;
use renderers::renderer2d::{self, Renderer2d, Transform};
use wgpu::{
    BlendState, Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceConfiguration, TextureView,
};
use wgpu_utils::mesh::{Mesh};
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

struct Sprite {
    mesh: Arc<Mesh<Vertex2DTextureCoordinateColor>>,
    texture: Arc<Texture>,
    affine: MovingAffine2,
    transform: renderer2d::Transform,
}

impl Sprite {
    fn update(&mut self, queue: &Queue, duration: Duration) {
        self.affine.update(duration);
        self.transform
            .enqueue_update(queue, self.affine.affine().into());
    }

    fn draw(&self, r: &mut renderer2d::RenderPass<'_>) {
        r.draw(&self.mesh, &self.texture, &self.transform);
    }
}

struct Text {
    font: Arc<Mutex<TextureAtlasFont>>,
    meshes_and_textures: Vec<(Mesh<Vertex2DTextureCoordinateColor>, Arc<Texture>)>,
    transform: renderer2d::Transform,
}

impl Text {
    pub fn new(device: &Device, font: Arc<Mutex<TextureAtlasFont>>) -> Self {
        Self {
            font,
            meshes_and_textures: Vec::new(),
            transform: Transform::new(device, Mat4::IDENTITY),
        }
    }

    pub fn enqueue_text_update(&mut self, device: &Device, s: &str) -> Result<()> {
        /*
        TODO re-use existing meshes if possible instead of always creating new ones
        TODO use a single mesh with different offsets for each texture?
        */
        let mut font = self.font.lock().unwrap();
        let layout = font.layout(s)?;
        self.meshes_and_textures.clear();
        for per_texture in layout.layout.iter() {
            let mut mesh_builder = MeshBuilder::<Vertex2DTextureCoordinateColor>::new();
            for glyph in per_texture.glyphs.iter() {
                mesh_builder.rectangle(
                    glyph.pixel_bounds,
                    glyph.texture_coordinate_bounds,
                    Color::WHITE,
                );
            }
            let mesh = mesh_builder.create_mesh(device);
            self.meshes_and_textures
                .push((mesh, per_texture.texture.clone()));
        }

        Ok(())
    }

    pub fn enqueue_transform_update(&mut self, queue: &Queue, affine: Affine2) {
        self.transform.enqueue_update(queue, affine.into());
    }

    pub fn draw(&self, r: &mut renderer2d::RenderPass) {
        for (mesh, texture) in self.meshes_and_textures.iter() {
            r.draw(mesh, texture, &self.transform);
        }
    }
}

struct Demo {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer_no_blend: Renderer2d,
    renderer_blend: Renderer2d,
    sprites: Vec<Sprite>,
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
        let sprite_texture = Arc::new(Texture::from_image(
            device.clone(),
            queue.clone(),
            Renderer2d::texture_bindings(),
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
                        rng.random_range(0.0..500.0),
                        rng.random_range(0.0..500.0),
                    ),
                    angular_velocity: rng
                        .random_range((-45.0f32.to_radians())..=(45.0f32.to_radians())),
                    velocity: Vec2::ZERO,
                },
                mesh: sprite_mesh.clone(),
                texture: sprite_texture.clone(),
                transform: renderer2d::Transform::new(&device, Mat4::zeroed()),
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
            Renderer2d::texture_bindings(),
            font.clone(),
            40.0,
        )?));

        let mut text = Text::new(&device, texture_atlas_font.clone());
        text.enqueue_transform_update(
            &queue,
            glam::Affine2::from_translation(Vec2::new(50.0, 50.0)).into(),
        );

        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            renderer_no_blend: Renderer2d::new(
                device.clone(),
                queue.clone(),
                surface_configuration,
                BlendState::REPLACE,
            ),
            renderer_blend: Renderer2d::new(
                device.clone(),
                queue.clone(),
                surface_configuration,
                BlendState::ALPHA_BLENDING,
            ),
            sprites,
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
                // TODO the sprite space should be the same aspect ratio as the screen, but some other coordinate system?

                let mut r = self
                    .renderer_no_blend
                    .render_pass(&mut render_pass, self.ortho);

                for sprite in self.sprites.iter() {
                    sprite.draw(&mut r);
                }
            }

            {
                let mut r = self
                    .renderer_blend
                    .render_pass(&mut render_pass, self.ortho);

                self.text.draw(&mut r);
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

        self.text.enqueue_text_update(
            &self.device,
            &format!("FPS: {}\nanother line, yjpqg", self.fps.fps_pretty()),
        )?;

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
