mod error;
mod graphics;

use bytemuck::{Pod, Zeroable};
use error::Error;
use glam::{Vec2, Vec4};
use gloo::{
    events::EventListener,
    render::{request_animation_frame, AnimationFrame},
    utils::{body, document, window},
};
use graphics::{
    array_buffer::ArrayBuffer,
    buffer_usage::BufferUsage,
    element_array_buffer::ElementArrayBuffer,
    shader::{AttributePointer, ShaderProgram},
};
use js_sys::wasm_bindgen::JsCast;
use log::*;
use serde::Serialize;
use std::{
    mem::offset_of,
    panic,
    rc::Rc,
    sync::{Arc, Mutex},
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec2,
}

struct State {
    context: Arc<WebGl2RenderingContext>,
    shader: ShaderProgram,
    position_attribute: AttributePointer,
    array_buffer: ArrayBuffer<Vertex>,
    element_aray_buffer: ElementArrayBuffer,
}

impl State {
    pub async fn new(context: Arc<WebGl2RenderingContext>) -> Result<State, Error> {
        let shader = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader.vertex.glsl"),
            include_str!("shaders/shader.fragment.glsl"),
        )?;

        let position_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("position_attribute")
                .ok_or("failed to find position attribute")?
                .clone(),
            2,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, position) as i32,
        );

        let array_buffer = ArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[
                Vertex {
                    position: Vec2 { x: -0.5, y: -0.5 },
                },
                Vertex {
                    position: Vec2 { x: 0.5, y: -0.5 },
                },
                Vertex {
                    position: Vec2 { x: 0.5, y: 0.5 },
                },
                Vertex {
                    position: Vec2 { x: -0.5, y: 0.5 },
                },
            ],
        )?;

        let element_aray_buffer = ElementArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[0, 1, 2, 2, 3, 0],
        )?;

        Ok(State {
            context,
            shader,
            position_attribute,
            array_buffer,
            element_aray_buffer,
        })
    }

    pub fn resize(&mut self, width: f64, height: f64) -> Result<(), Error> {
        self.context
            .viewport(0, 0, width.floor() as i32, height.floor() as i32);
        Ok(())
    }

    pub fn anim(&mut self, time: f64) -> Result<(), Error> {
        self.context
            .clear_color(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader.use_program();

        self.array_buffer.bind();
        self.element_aray_buffer.bind();

        self.position_attribute.enable();

        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.element_aray_buffer.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        self.position_attribute.disable();

        self.array_buffer.bind_none();
        self.element_aray_buffer.bind_none();

        self.shader.use_none();

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).map_err(|e| e.to_string())?;

    let canvas: HtmlCanvasElement = document()
        .create_element("canvas")?
        .dyn_into()
        .map_err(|_| "failed to get canvas as the right type of element")?;
    canvas.style().set_property("position", "absolute")?;
    canvas.style().set_property("left", "0")?;
    canvas.style().set_property("top", "0")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    body().replace_children_with_node_1(&canvas);

    #[derive(Serialize)]
    struct WebGLOptions {
        #[serde(rename = "powerPreference")]
        power_preference: String,
    }
    let context: WebGl2RenderingContext = canvas
        .get_context_with_context_options(
            "webgl2",
            &serde_wasm_bindgen::to_value(&WebGLOptions {
                power_preference: "high-performance".to_owned(),
            })
            .map_err(|e| format!("failed to serialize webgl options: {e:?}"))?,
        )?
        .ok_or("failed to create webgl context")?
        .dyn_into()
        .map_err(|e| {
            format!("created a canvas graphics context, but it wasn't the expected type: {e:?}")
        })?;

    let canvas = Rc::new(canvas);
    let state: Rc<Mutex<Option<State>>> = Rc::new(Mutex::new(None));

    {
        let canvas = canvas.clone();
        let state = state.clone();
        spawn_local(async move {
            match State::new(Arc::new(context)).await {
                Ok(mut s) => {
                    if let Err(e) = resize(&canvas, &mut s) {
                        panic!("initial resize error: {e:?}");
                    }
                    let state = &mut *state.lock().unwrap();
                    state.replace(s);
                }
                Err(e) => panic!("error initializing: {e:?}"),
            }
        });
    }

    {
        let canvas = canvas.clone();
        let state = state.clone();
        EventListener::new(&window(), "resize", move |_| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Err(e) = resize(&canvas, state) {
                    error!("error resizing: {e:?}");
                }
            }
        })
        .forget();
    }

    {
        let state = state.clone();
        anim_loop(move |time| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Err(e) = state.anim(time) {
                    error!("error running animation loop: {e:?}");
                }
            }
        });
    }

    Ok(())
}

fn resize(canvas: &HtmlCanvasElement, state: &mut State) -> Result<(), Error> {
    let width = window().inner_width()?.as_f64().ok_or("expected float")?;
    let height = window().inner_height()?.as_f64().ok_or("expected float")?;
    canvas.set_width(width.floor() as u32);
    canvas.set_height(height.floor() as u32);
    state.resize(width, height)
}

fn anim_loop<F: Fn(f64) + 'static>(f: F) {
    fn inner<F: Fn(f64) + 'static>(last_anim: Rc<Mutex<Option<AnimationFrame>>>, f: F) {
        let callback = {
            let last_anim = last_anim.clone();
            move |time| {
                f(time);
                inner(last_anim, f);
            }
        };

        {
            let last_anim = &mut *last_anim.lock().unwrap();
            last_anim.replace(request_animation_frame(callback));
        }
    }

    inner(Rc::new(Mutex::new(None)), f);
}
