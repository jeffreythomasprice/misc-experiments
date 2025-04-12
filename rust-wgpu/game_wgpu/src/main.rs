mod app;
mod graphics_utils;
mod wgpu_utils;

use std::cell::RefCell;
use std::time::Duration;
use std::{f32::consts::TAU, sync::Arc};

use app::{App, Renderer};
use bytemuck::Zeroable;
use color_eyre::eyre::{Result, eyre};
use glam::{Mat4, UVec2, Vec2};
use graphics_utils::basic_types::{Affine2, RGBA, Rect, Vertex2DTextureCoordinateRGBA};
use graphics_utils::fps::FPSCounter;
use graphics_utils::mesh_builder::MeshBuilder;
use graphics_utils::pipelines::pipeline_2d_textured::{
    self, Pipeline2DTextured, Pipeline2DTexturedOptions,
};
use image::ImageBuffer;
use rand::Rng;
use tracing::*;
use wgpu::{
    BlendState, BufferUsages, Color, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, SurfaceConfiguration, TextureView,
};
use wgpu_utils::mesh::Mesh;
use wgpu_utils::texture::Texture;
use wgpu_utils::{buffer::Buffer, font::Font};
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

/*
TODO more wgpu stuff
https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/
*/

trait Renderable<Renderer> {
    fn render(&self, renderer: &mut Renderer);
}

struct MeshAndUniforms<Vertex> {
    mesh: Arc<RefCell<Mesh<Vertex>>>,
    texture: Arc<RefCell<Texture>>,
    uniform_buffer: pipeline_2d_textured::ModelUniform,
}

impl MeshAndUniforms<Vertex2DTextureCoordinateRGBA> {
    pub fn new(
        pipeline: &Pipeline2DTextured,
        mesh: Arc<RefCell<Mesh<Vertex2DTextureCoordinateRGBA>>>,
        texture: Arc<RefCell<Texture>>,
    ) -> Self {
        Self {
            mesh,
            texture,
            uniform_buffer: pipeline.new_model_uniform(),
        }
    }
}

impl Renderable<pipeline_2d_textured::Renderer<'_>>
    for MeshAndUniforms<Vertex2DTextureCoordinateRGBA>
{
    fn render(&self, renderer: &mut pipeline_2d_textured::Renderer) {
        renderer.render(
            &self.texture.borrow(),
            &self.uniform_buffer,
            &self.mesh.borrow(),
        );
    }
}

trait Updatable {
    fn update(&mut self, duration: Duration) -> Result<()>;
}

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
}

impl Updatable for MovingAffine2 {
    fn update(&mut self, duration: Duration) -> Result<()> {
        // TODO math helper for wrapping a value to inside range?
        let new_angle =
            (self.angle + self.angular_velocity * duration.as_secs_f32()) % std::f32::consts::TAU;
        self.angle = if new_angle >= 0.0 {
            new_angle
        } else {
            std::f32::consts::TAU - new_angle
        };

        self.translation += self.velocity * duration.as_secs_f32();
        Ok(())
    }
}

struct Model {
    renderable: MeshAndUniforms<Vertex2DTextureCoordinateRGBA>,
    transform: MovingAffine2,
}

impl Renderable<pipeline_2d_textured::Renderer<'_>> for Model {
    fn render(&self, renderer: &mut pipeline_2d_textured::Renderer) {
        self.renderable.render(renderer);
    }
}

impl Updatable for Model {
    fn update(&mut self, duration: Duration) -> Result<()> {
        self.transform.update(duration)?;
        self.renderable
            .uniform_buffer
            .enqueue_update(self.transform.affine().into());
        Ok(())
    }
}

struct TextureFontRenderable {
    device: Arc<Device>,
    queue: Arc<Queue>,
    font: Arc<Font<'static>>,
    texture: Arc<RefCell<Texture>>,
    mesh: Arc<RefCell<Mesh<Vertex2DTextureCoordinateRGBA>>>,
    renderable: MeshAndUniforms<Vertex2DTextureCoordinateRGBA>,
}

impl TextureFontRenderable {
    pub fn new(pipeline: &Pipeline2DTextured, font: Arc<Font<'static>>) -> Result<Self> {
        let texture = Arc::new(RefCell::new(Texture::new(
            pipeline.device().clone(),
            pipeline.queue().clone(),
            Pipeline2DTextured::TEXTURE_BINDING,
            Pipeline2DTextured::TEXTURE_SAMPLER_BINDING,
            // initial contents don't matter, re'll be re-creating this as needed to make it big
            image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
                1,
                1,
                image::Rgba([0, 0, 0, 0]),
            )),
        )?));
        let mesh = Arc::new(RefCell::new(Mesh::new(
            Buffer::new_init(
                &pipeline.device(),
                None,
                // initial contents don't matter, just make sure it's big enough
                &[Vertex2DTextureCoordinateRGBA::zeroed(); 4],
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            ),
            Buffer::new_init(
                &pipeline.device(),
                None,
                &[0, 1, 2, 2, 3, 0],
                BufferUsages::INDEX | BufferUsages::COPY_DST,
            ),
        )));
        let mut renderable = MeshAndUniforms::new(pipeline, mesh.clone(), texture.clone());
        renderable.uniform_buffer.enqueue_update(Mat4::IDENTITY);
        Ok(Self {
            device: pipeline.device().clone(),
            queue: pipeline.queue().clone(),
            font,
            texture,
            mesh,
            renderable,
        })
    }

    pub fn set_text(&mut self, s: &str) -> Result<()> {
        let mut texture = self.texture.borrow_mut();
        let mut mesh = self.mesh.borrow_mut();

        let (font_image, _font_bounding_box) = self.font.render_to_new_image(s, 40.0);

        // update texture
        let font_image_width = font_image.width();
        let font_image_height = font_image.height();
        if font_image_width > texture.width() || font_image_height > texture.height() {
            // texture is too small to fit image, recreate
            *texture = Texture::new(
                self.device.clone(),
                self.queue.clone(),
                Pipeline2DTextured::TEXTURE_BINDING,
                Pipeline2DTextured::TEXTURE_SAMPLER_BINDING,
                font_image,
            )?;
        } else {
            // texture is big enough already, just copy the new image into it
            texture.enqueue_update(font_image, UVec2::ZERO);
        }

        // update mesh
        MeshBuilder::new()
            .rectangle(
                Rect::from_origin_size(
                    Vec2::new(0.0, 0.0),
                    Vec2::new(texture.width() as f32, texture.height() as f32),
                ),
                Rect {
                    min: Vec2::new(0.0, 0.0),
                    max: Vec2::new(
                        (font_image_width as f32) / (texture.width() as f32),
                        (font_image_height as f32) / (texture.height() as f32),
                    ),
                },
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            )
            .enqueue_update(&self.queue, &mut mesh);

        Ok(())
    }

    pub fn set_affine2(&mut self, a: Affine2) {
        self.renderable.uniform_buffer.enqueue_update(a.into());
    }
}

impl Renderable<pipeline_2d_textured::Renderer<'_>> for TextureFontRenderable {
    fn render(&self, renderer: &mut pipeline_2d_textured::Renderer) {
        self.renderable.render(renderer);
    }
}

struct Demo {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline_no_blending: Pipeline2DTextured,
    pipeline_blending: Pipeline2DTextured,
    models: Vec<Model>,
    font_string: TextureFontRenderable,
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

        let texture = Arc::new(RefCell::new(Texture::new(
            device.clone(),
            queue.clone(),
            Pipeline2DTextured::TEXTURE_BINDING,
            Pipeline2DTextured::TEXTURE_SAMPLER_BINDING,
            image::load_from_memory_with_format(
                include_bytes!("../assets/rustacean-flat-happy.png"),
                image::ImageFormat::Png,
            )?,
        )?));
        info!("texture size: {:?}", texture.borrow().size());

        let mesh = {
            let texture = texture.borrow();
            Arc::new(RefCell::new(
                MeshBuilder::new()
                    .rectangle(
                        Rect::from_origin_size(
                            Vec2::new(
                                -(texture.width() as f32 / 2.0),
                                -(texture.height() as f32 / 2.0),
                            ),
                            Vec2::new(texture.width() as f32, texture.height() as f32),
                        ),
                        Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                        RGBA {
                            red: 1.0,
                            green: 1.0,
                            blue: 1.0,
                            alpha: 1.0,
                        },
                    )
                    .create_mesh(&device),
            ))
        };

        let mut rng = rand::rng();
        let mut models = Vec::new();
        for _ in 0..10 {
            let scale = rng.random_range(0.75..1.25);
            models.push(Model {
                renderable: MeshAndUniforms::new(
                    &pipeline_no_blending,
                    mesh.clone(),
                    texture.clone(),
                ),
                transform: MovingAffine2 {
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
            });
        }

        let font = Arc::new(Font::new(
            rusttype::Font::try_from_bytes(include_bytes!(
                "../assets/calibri-font-family/calibri-regular.ttf"
            ))
            .ok_or(eyre!("failed to parse font"))?,
        ));
        let mut font_string = TextureFontRenderable::new(&pipeline_blending, font.clone())?;
        font_string.set_affine2(glam::Affine2::from_translation(Vec2::new(50.0, 100.0)).into());

        Ok(Self {
            device,
            queue,
            pipeline_no_blending,
            pipeline_blending,
            models,
            font_string,
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
        self.font_string
            .set_text(&format!("FPS: {}", self.fps.fps_pretty()))?;

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
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
                    for model in self.models.iter() {
                        model.render(r);
                    }
                });
            self.pipeline_blending.render(render_pass, self.ortho, |r| {
                self.font_string.render(r);
            });
        }
        self.queue.submit([encoder.finish()]);

        Ok(())
    }

    fn update(&mut self, duration: Duration) -> Result<()> {
        self.fps.tick(duration);

        for model in self.models.iter_mut() {
            model.update(duration)?;
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
