mod dom;
mod events;
mod geom;
mod graphics;

use anyhow::{anyhow, Result};
use bytemuck::{offset_of, Pod, Zeroable};
use events::EventHandler;
use graphics::{
    array_buffer::ArrayBuffer,
    shader::{AttributePointer, ShaderProgram},
};
use log::*;
use nalgebra::Matrix4;
use nalgebra_glm::{ortho, Vec2};
use std::{panic, sync::Arc, time::Duration};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct RGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec2,
    color: RGBA,
}

struct DemoState {
    context: Arc<WebGl2RenderingContext>,
    shader: ShaderProgram,
    position_attribute: AttributePointer,
    color_attribute: AttributePointer,
    array_buffer: ArrayBuffer<Vertex>,

    projection_matrix: Matrix4<f32>,
}

impl DemoState {
    pub fn new(context: Arc<WebGl2RenderingContext>) -> Result<Self> {
        context.clear_color(0.25, 0.5, 1.0, 1.0);

        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("./demo/shaders/demo-vertex.glsl"),
            include_str!("./demo/shaders/demo-fragment.glsl"),
        )?;

        let position_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("position_attribute")
                .ok_or(anyhow!("failed to find position attribute"))?
                .clone(),
            2,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, position) as i32,
        );

        let color_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("color_attribute")
                .ok_or(anyhow!("failed to find color attribute"))?
                .clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, color) as i32,
        );

        let array_buffer = ArrayBuffer::new_with_data(
            context.clone(),
            graphics::array_buffer::Usage::DynamicDraw,
            &[
                Vertex {
                    position: Vec2::new(50.0, 50.0),
                    color: RGBA {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec2::new(300.0, 50.0),
                    color: RGBA {
                        red: 0.0,
                        green: 1.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec2::new(50.0, 300.0),
                    color: RGBA {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ],
        )?;

        Ok(Self {
            context,
            shader,
            position_attribute,
            color_attribute,
            array_buffer,

            projection_matrix: Matrix4::identity(),
        })
    }
}

impl EventHandler for DemoState {
    fn deactivate(&mut self) -> Result<()> {
        Ok(())
    }

    fn resize(&mut self, size: geom::Size<u32>) -> Result<events::NextEventHandler> {
        self.context
            .viewport(0, 0, size.width as i32, size.height as i32);

        self.projection_matrix =
            ortho::<f32>(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);

        Ok(events::NextEventHandler::NoChange)
    }

    fn render(&mut self) -> Result<events::NextEventHandler> {
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader.use_program();

        // TODO set projection matrix
        let projection_matrix_uniform = self
            .shader
            .get_uniform_by_name("projection_matrix_uniform")
            .ok_or(anyhow!("failed to find projection matrix uniform"))?;
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&projection_matrix_uniform.location),
            false,
            self.projection_matrix.as_slice(),
        );

        self.array_buffer.bind();

        self.position_attribute.enable();
        self.color_attribute.enable();
        self.context.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            0,
            self.array_buffer.len() as i32,
        );
        self.position_attribute.disable();
        self.color_attribute.disable();

        self.array_buffer.bind_none();
        self.shader.use_none();

        Ok(events::NextEventHandler::NoChange)
    }

    fn update(&mut self, delta: Duration) -> Result<events::NextEventHandler> {
        Ok(events::NextEventHandler::NoChange)
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        match events::run(Box::new(|context| match DemoState::new(context) {
            Ok(result) => Ok(Box::new(result)),
            Err(e) => Err(e),
        }))
        .await
        {
            Ok(_) => {
                warn!("state machine exited without error, but really it should keep going forever so something is probably wrong");
            }
            Err(e) => error!("state machine exited with: {e:?}"),
        };
    });
}
