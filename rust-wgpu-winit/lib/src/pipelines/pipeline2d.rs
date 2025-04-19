use wgpu::include_wgsl;

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, FragmentState, FrontFace, IndexFormat,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor,
    SurfaceConfiguration, VertexState,
};

use crate::{
    basic_types::{HasVertexBufferLayout, Vertex2DTextureCoordinateColor},
    {
        mesh::Mesh,
        texture::{self, Texture, TextureBindings},
        uniform_buffer::{self, UniformBuffer},
    },
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

pub struct Transform {
    uniform_buffer: UniformBuffer<ModelUniformData>,
}

impl Transform {
    pub fn new(device: &Device, initial_value: Mat4) -> Self {
        Self {
            uniform_buffer: UniformBuffer::new_init(
                device,
                ModelUniformData {
                    modelview_matrix: initial_value,
                },
                Pipeline2d::MODEL_UNIFORM_BINDING,
            ),
        }
    }

    pub fn enqueue_update(&mut self, queue: &Queue, m: Mat4) {
        self.uniform_buffer.enqueue_update(
            queue,
            ModelUniformData {
                modelview_matrix: m,
            },
        );
    }
}

pub struct RenderPass<'a> {
    queue: &'a Queue,
    render_pass: &'a mut wgpu::RenderPass<'static>,
}

impl<'a> RenderPass<'a> {
    pub fn draw(
        &mut self,
        mesh: &Mesh<Vertex2DTextureCoordinateColor>,
        texture: &Texture,
        transform: &Transform,
    ) {
        self.render_pass
            .set_bind_group(1, transform.uniform_buffer.bind_group(), &[]);
        self.render_pass
            .set_bind_group(2, texture.bind_group(), &[]);
        self.render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer().buffer().slice(..));
        self.render_pass
            .set_index_buffer(mesh.index_buffer().buffer().slice(..), IndexFormat::Uint16);
        self.render_pass
            .draw_indexed(0..(mesh.index_buffer().len() as u32), 0, 0..1);
    }
}

pub struct Pipeline2d {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: RenderPipeline,
    scene_uniform_buffer: UniformBuffer<SceneUniformData>,
}

impl Pipeline2d {
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
        blend_state: BlendState,
    ) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("./pipeline2d.wsgl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &uniform_buffer::bind_group_layout(&device, Self::SCENE_UNIFORM_BINDING),
                    &uniform_buffer::bind_group_layout(&device, Self::MODEL_UNIFORM_BINDING),
                    &texture::bind_group_layout(
                        &device,
                        Self::TEXTURE_BINDING,
                        Self::TEXTURE_SAMPLER_BINDING,
                    ),
                ],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex2DTextureCoordinateColor::vertex_buffer_layout()],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
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
        });
        let scene_uniform_buffer = UniformBuffer::new_init(
            &device,
            SceneUniformData::zeroed(),
            Self::SCENE_UNIFORM_BINDING,
        );
        Self {
            device,
            queue,
            pipeline,
            scene_uniform_buffer,
        }
    }

    pub fn render_pass<'a>(
        &'a mut self,
        render_pass: &'a mut wgpu::RenderPass<'static>,
        projection_matrix: Mat4,
    ) -> RenderPass<'a> {
        self.scene_uniform_buffer
            .enqueue_update(&self.queue, SceneUniformData { projection_matrix });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.scene_uniform_buffer.bind_group(), &[]);

        RenderPass {
            queue: &self.queue,
            render_pass,
        }
    }
}
