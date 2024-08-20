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
use std::{panic, process, sync::Arc, time::Duration};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    // TODO points?
    x: f32,
    y: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct DemoState {
    context: Arc<WebGl2RenderingContext>,
    shader: ShaderProgram,
    position_attribute: AttributePointer,
    color_attribute: AttributePointer,
    array_buffer: ArrayBuffer<Vertex>,
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
            offset_of!(Vertex, x) as i32,
        );

        let color_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("color_attribute")
                .ok_or(anyhow!("failed to find color attribute"))?
                .clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, r) as i32,
        );

        let mut array_buffer = ArrayBuffer::new(
            context.clone(),
            graphics::array_buffer::Usage::DynamicDraw,
            3,
        )?;
        array_buffer.set(
            &[
                Vertex {
                    x: -0.5,
                    y: -0.5,
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                Vertex {
                    x: 0.5,
                    y: -0.5,
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 0.0,
                },
                Vertex {
                    x: 0.0,
                    y: 0.5,
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                },
            ],
            0,
        )?;

        Ok(Self {
            context,
            shader,
            position_attribute,
            color_attribute,
            array_buffer,
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

        Ok(events::NextEventHandler::NoChange)
    }

    fn render(&mut self) -> Result<events::NextEventHandler> {
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader.use_program();
        self.array_buffer.bind();

        self.position_attribute.enable();
        self.color_attribute.enable();
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
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
