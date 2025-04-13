use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, FragmentState, FrontFace, IndexFormat,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    SurfaceConfiguration, VertexState, include_wgsl,
};

use crate::{
    graphics_utils::basic_types::{HasVertexBufferLayout, Vertex2DTextureCoordinateColor},
    wgpu_utils::{
        mesh::Mesh,
        texture::{self, Texture, TextureBindings},
        uniform_buffer::{self, UniformBuffer},
    },
};

use super::{HasDeviceAndQueue, HasTextureBindings};

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

pub struct ModelUniform {
    queue: Arc<Queue>,
    buffer: UniformBuffer<ModelUniformData>,
}

impl ModelUniform {
    pub fn enqueue_update(&mut self, modelview_matrix: Mat4) {
        self.buffer
            .enqueue_update(&self.queue, ModelUniformData { modelview_matrix });
    }
}

pub struct Renderer<'a> {
    render_pass: RenderPass<'a>,
}

impl<'a> Renderer<'a> {
    pub fn render(
        &mut self,
        texture: &Texture,
        uniform: &ModelUniform,
        mesh: &Mesh<Vertex2DTextureCoordinateColor>,
    ) {
        self.render_pass
            .set_bind_group(1, uniform.buffer.bind_group(), &[]);
        self.render_pass
            .set_bind_group(2, texture.bind_group(), &[]);
        self.render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer().buffer().slice(..));
        self.render_pass
            .set_index_buffer(mesh.index_buffer().buffer().slice(..), IndexFormat::Uint16);
        self.render_pass
            .draw_indexed(0..(mesh.index_buffer().len() as u32), 0, 0..1);
    }

    pub fn finish(self) -> RenderPass<'a> {
        self.render_pass
    }
}

pub struct Pipeline2DTexturedOptions {
    pub blend_state: BlendState,
}

pub struct Pipeline2DTextured {
    device: Arc<Device>,
    queue: Arc<Queue>,
    scene_uniform: UniformBuffer<SceneUniformData>,
    render_pipeline: RenderPipeline,
}

impl Pipeline2DTextured {
    const SCENE_UNIFORM_BINDING: u32 = 0;
    const MODEL_UNIFORM_BINDING: u32 = 0;
    const TEXTURE_BINDING: u32 = 0;
    const TEXTURE_SAMPLER_BINDING: u32 = 1;

    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface_configuration: &SurfaceConfiguration,
        options: Pipeline2DTexturedOptions,
    ) -> Self {
        let scene_uniform = UniformBuffer::new_init(
            &device,
            SceneUniformData::zeroed(),
            Self::SCENE_UNIFORM_BINDING,
        );

        let shader_module =
            device.create_shader_module(include_wgsl!("./pipeline_2d_textured.wsgl"));
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    blend: Some(options.blend_state),
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

        Self {
            device,
            queue,
            scene_uniform,
            render_pipeline,
        }
    }

    pub fn new_model_uniform(&self) -> ModelUniform {
        ModelUniform {
            queue: self.queue.clone(),
            buffer: UniformBuffer::new_init(
                &self.device,
                Zeroable::zeroed(),
                Self::MODEL_UNIFORM_BINDING,
            ),
        }
    }

    pub fn render<'a, 'b: 'a>(
        &'b mut self,
        mut render_pass: RenderPass<'a>,
        projection_matrix: Mat4,
        f: impl Fn(&mut Renderer<'a>),
    ) -> RenderPass<'a> {
        self.scene_uniform
            .enqueue_update(&self.queue, SceneUniformData { projection_matrix });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.scene_uniform.bind_group(), &[]);

        let mut r = Renderer { render_pass };
        f(&mut r);
        r.finish()
    }
}

impl HasDeviceAndQueue for Pipeline2DTextured {
    fn device(&self) -> &Arc<Device> {
        &self.device
    }

    fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }
}

impl HasTextureBindings for Pipeline2DTextured {
    fn texture_binding() -> TextureBindings {
        TextureBindings {
            texture: Self::TEXTURE_BINDING,
            sampler: Self::TEXTURE_SAMPLER_BINDING,
        }
    }
}
