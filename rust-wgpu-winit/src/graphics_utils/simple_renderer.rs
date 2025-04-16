use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use glam::Mat4;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, FragmentState, FrontFace, IndexFormat,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, SurfaceConfiguration, VertexState, include_wgsl,
};

use crate::{
    misc_utils::pool::{Arena, Pool},
    wgpu_utils::{
        mesh::Mesh,
        texture::{self, Texture, TextureBindings},
        uniform_buffer::{self, UniformBuffer},
    },
};

use super::{
    basic_types::{Affine2, HasVertexBufferLayout, Rect, Vertex2DTextureCoordinateColor},
    colors::Color,
    mesh_builder::MeshBuilder,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct SceneUniformData {
    projection_matrix: Mat4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ModelUniformData {
    modelview_matrix: Mat4,
}

pub struct RenderPass<'a> {
    queue: &'a Queue,
    render_pass: &'a mut wgpu::RenderPass<'static>,
    pipeline_no_blending: &'a RenderPipeline,
    pipeline_blending: &'a RenderPipeline,
    scene_uniform_buffer: &'a UniformBuffer<SceneUniformData>,
    model_uniform_buffer_arena: Arena<UniformBuffer<ModelUniformData>>,
    quad_mesh_arena: Arena<Mesh<Vertex2DTextureCoordinateColor>>,
    blend: bool,
}

impl<'a> RenderPass<'a> {
    fn new(
        queue: &'a Queue,
        render_pass: &'a mut wgpu::RenderPass<'static>,
        pipeline_no_blending: &'a RenderPipeline,
        pipeline_blending: &'a RenderPipeline,
        scene_uniform_buffer: &'a UniformBuffer<SceneUniformData>,
        model_uniform_buffer_arena: Arena<UniformBuffer<ModelUniformData>>,
        quad_mesh_arena: Arena<Mesh<Vertex2DTextureCoordinateColor>>,
    ) -> Self {
        let mut result = Self {
            queue,
            render_pass,
            pipeline_no_blending,
            pipeline_blending,
            scene_uniform_buffer,
            model_uniform_buffer_arena,
            quad_mesh_arena,
            blend: false,
        };
        result.update_pipeline();
        result
    }

    pub fn blend(&self) -> bool {
        self.blend
    }

    pub fn set_blend(&mut self, b: bool) {
        if b != self.blend {
            self.blend = b;
            self.update_pipeline();
        }
    }

    pub fn fill_rect_texture(
        &mut self,
        transform: Affine2,
        bounds: Rect,
        texture: &Texture,
        texture_coordinates: Rect,
        color: Color,
    ) -> Result<()> {
        let model_uniform_buffer = self.model_uniform_buffer_arena.get_mut()?;
        model_uniform_buffer.enqueue_update(
            self.queue,
            ModelUniformData {
                modelview_matrix: transform.into(),
            },
        );

        let mesh: &mut Mesh<Vertex2DTextureCoordinateColor> = self.quad_mesh_arena.get_mut()?;
        let mut mesh_builder = MeshBuilder::<Vertex2DTextureCoordinateColor>::new();
        mesh_builder.rectangle(bounds, texture_coordinates, color);
        mesh_builder.enqueue_update(self.queue, mesh);

        self.render_pass
            .set_bind_group(1, model_uniform_buffer.bind_group(), &[]);
        self.render_pass
            .set_bind_group(2, texture.bind_group(), &[]);
        self.render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer().buffer().slice(..));
        self.render_pass
            .set_index_buffer(mesh.index_buffer().buffer().slice(..), IndexFormat::Uint16);
        self.render_pass
            .draw_indexed(0..(mesh.index_buffer().len() as u32), 0, 0..1);

        Ok(())
    }

    fn update_pipeline(&mut self) {
        self.render_pass.set_pipeline(if self.blend {
            self.pipeline_blending
        } else {
            self.pipeline_no_blending
        });
        self.render_pass
            .set_bind_group(0, self.scene_uniform_buffer.bind_group(), &[]);
    }
}

pub struct SimpleRenderer {
    queue: Arc<Queue>,
    pipeline_no_blending: RenderPipeline,
    pipeline_blending: RenderPipeline,
    scene_uniform_buffer: UniformBuffer<SceneUniformData>,
    model_uniform_buffer_pool: Pool<UniformBuffer<ModelUniformData>>,
    quad_mesh_pool: Pool<Mesh<Vertex2DTextureCoordinateColor>>,
    viewport: Rect,
    ortho: Mat4,
}

impl SimpleRenderer {
    const SCENE_UNIFORM_BINDING: u32 = 0;
    const MODEL_UNIFORM_BINDING: u32 = 0;
    const TEXTURE_BINDING: u32 = 0;
    const TEXTURE_SAMPLER_BINDING: u32 = 1;

    pub fn texture_bindings() -> TextureBindings {
        TextureBindings {
            texture: Self::TEXTURE_BINDING,
            sampler: Self::TEXTURE_SAMPLER_BINDING,
        }
    }

    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("./simple_renderer.wsgl"));
        Self {
            queue,
            pipeline_no_blending: Self::create_render_pipeline(
                &device,
                surface_configuration,
                &shader_module,
                BlendState::REPLACE,
            ),
            pipeline_blending: Self::create_render_pipeline(
                &device,
                surface_configuration,
                &shader_module,
                BlendState::ALPHA_BLENDING,
            ),
            scene_uniform_buffer: UniformBuffer::new_init(
                &device,
                SceneUniformData::zeroed(),
                Self::SCENE_UNIFORM_BINDING,
            ),
            model_uniform_buffer_pool: {
                let device = device.clone();
                Pool::new(move || {
                    Ok(UniformBuffer::new_init(
                        &device,
                        Zeroable::zeroed(),
                        Self::MODEL_UNIFORM_BINDING,
                    ))
                })
            },
            quad_mesh_pool: {
                let device = device.clone();
                Pool::new(move || {
                    let mut builder = MeshBuilder::<Vertex2DTextureCoordinateColor>::new();
                    builder.rectangle(Rect::zeroed(), Rect::zeroed(), Color::zeroed());
                    Ok(builder.create_mesh(&device))
                })
            },
            viewport: Rect::zeroed(),
            ortho: Mat4::zeroed(),
        }
    }

    pub fn render_pass<'a>(
        &'a mut self,
        render_pass: &'a mut wgpu::RenderPass<'static>,
    ) -> RenderPass<'a> {
        self.scene_uniform_buffer.enqueue_update(
            &self.queue,
            SceneUniformData {
                projection_matrix: self.ortho,
            },
        );
        RenderPass::new(
            &self.queue,
            render_pass,
            &self.pipeline_no_blending,
            &self.pipeline_blending,
            &self.scene_uniform_buffer,
            self.model_uniform_buffer_pool.arena(),
            self.quad_mesh_pool.arena(),
        )
    }

    pub fn viewport(&self) -> Rect {
        self.viewport
    }

    pub fn set_viewport(&mut self, r: Rect) {
        self.viewport = r;
        self.ortho = Mat4::orthographic_rh_gl(r.min.x, r.max.x, r.max.y, r.min.y, -1.0, 1.0);
    }

    fn create_render_pipeline(
        device: &Device,
        surface_configuration: &SurfaceConfiguration,
        shader_module: &ShaderModule,
        blend_state: BlendState,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &uniform_buffer::bind_group_layout(device, Self::SCENE_UNIFORM_BINDING),
                    &uniform_buffer::bind_group_layout(device, Self::MODEL_UNIFORM_BINDING),
                    &texture::bind_group_layout(
                        device,
                        Self::TEXTURE_BINDING,
                        Self::TEXTURE_SAMPLER_BINDING,
                    ),
                ],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                module: shader_module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex2DTextureCoordinateColor::vertex_buffer_layout()],
            },
            fragment: Some(FragmentState {
                module: shader_module,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(blend_state),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }
}
