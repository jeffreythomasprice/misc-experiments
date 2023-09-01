#![feature(offset_of)]

use std::{collections::HashMap, rc::Rc, sync::Mutex};

use lib::{
    dom::{
        anim_frame::{request_animation_frame_loop, RequestAnimationFrameStatus},
        getters::{get_body, get_document, get_window},
    },
    errors::Result,
    glmath::{
        angles::{Degrees, Radians},
        matrix4::Matrix4,
        numbers::CouldBeAnAngle,
        rgba::Rgba,
        vector2::Vector2,
        vector3::Vector3,
    },
    webgl::{
        buffers::Buffer,
        shaders::ShaderProgram,
        textures::Texture,
        vertexarrays::{VertexArray, VertexArrayAttribute},
    },
};
use log::*;

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

#[repr(C)]
#[derive(Debug)]
struct Vertex {
    position: Vector2<f32>,
    texture_coordinate: Vector2<f32>,
    color: Rgba<f32>,
}

struct State {
    canvas: HtmlCanvasElement,
    context: Rc<WebGl2RenderingContext>,

    program: ShaderProgram,
    _buffer: Buffer,
    vertex_array: VertexArray,
    texture: Texture,

    last_time: f64,

    ortho_matrix: Matrix4<f32>,
    perspective_matrix: Matrix4<f32>,
    rotation: Degrees<f32>,
}

impl State {
    pub fn new(canvas: HtmlCanvasElement, context: WebGl2RenderingContext) -> Result<Self> {
        let context = Rc::new(context);

        let program = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader.vertex"),
            include_str!("shaders/shader.fragment"),
        )?;

        let position_attribute = program.get_attribute("in_position")?;
        let texture_coordinate_attribute = program.get_attribute("in_texture_coordinate")?;
        let color_attribute = program.get_attribute("in_color")?;

        let buffer = Buffer::new_with_typed(
            context.clone(),
            WebGl2RenderingContext::ARRAY_BUFFER,
            &[
                Vertex {
                    position: Vector2::new(-1f32, -1f32),
                    texture_coordinate: Vector2::new(0f32, 0f32),
                    color: Rgba::new(1f32, 1f32, 1f32, 1f32),
                },
                Vertex {
                    position: Vector2::new(1f32, -1f32),
                    texture_coordinate: Vector2::new(1f32, 0f32),
                    color: Rgba::new(1f32, 1f32, 1f32, 1f32),
                },
                Vertex {
                    position: Vector2::new(1f32, 1f32),
                    texture_coordinate: Vector2::new(1f32, 1f32),
                    color: Rgba::new(1f32, 1f32, 1f32, 1f32),
                },
                Vertex {
                    position: Vector2::new(1f32, 1f32),
                    texture_coordinate: Vector2::new(1f32, 1f32),
                    color: Rgba::new(1f32, 1f32, 1f32, 1f32),
                },
                Vertex {
                    position: Vector2::new(-1f32, 1f32),
                    texture_coordinate: Vector2::new(0f32, 1f32),
                    color: Rgba::new(1f32, 1f32, 1f32, 1f32),
                },
                Vertex {
                    position: Vector2::new(-1f32, -1f32),
                    texture_coordinate: Vector2::new(0f32, 0f32),
                    color: Rgba::new(1f32, 1f32, 1f32, 1f32),
                },
            ],
            WebGl2RenderingContext::STATIC_DRAW,
        )?;

        let vertex_array = VertexArray::new(
            context.clone(),
            &[
                VertexArrayAttribute {
                    shader_attribute: position_attribute,
                    buffer: &buffer,
                    size: 2,
                    type_: WebGl2RenderingContext::FLOAT,
                    normalized: false,
                    stride: std::mem::size_of::<Vertex>(),
                    offset: std::mem::offset_of!(Vertex, position),
                },
                VertexArrayAttribute {
                    shader_attribute: texture_coordinate_attribute,
                    buffer: &buffer,
                    size: 2,
                    type_: WebGl2RenderingContext::FLOAT,
                    normalized: false,
                    stride: std::mem::size_of::<Vertex>(),
                    offset: std::mem::offset_of!(Vertex, texture_coordinate),
                },
                VertexArrayAttribute {
                    shader_attribute: color_attribute,
                    buffer: &buffer,
                    size: 4,
                    type_: WebGl2RenderingContext::FLOAT,
                    normalized: false,
                    stride: std::mem::size_of::<Vertex>(),
                    offset: std::mem::offset_of!(Vertex, color),
                },
            ],
        )?;

        let texture_width = 256u32;
        let texture_height = 256u32;
        let mut pixels: Vec<Rgba<u8>> =
            Vec::with_capacity((texture_width as usize) * (texture_height as usize));
        for y in 0..texture_height {
            let a = ((y as f64) / (texture_height as f64) * 255f64) as u8;
            for x in 0..texture_width {
                let b = ((x as f64) / (texture_width as f64) * 255f64) as u8;
                pixels.push(Rgba::new(a, b, 255 - a, 255));
            }
        }
        let texture = unsafe {
            Texture::new_2d_rgba_u8(
                context.clone(),
                texture_width,
                texture_height,
                core::slice::from_raw_parts(
                    pixels.as_ptr() as *const u8,
                    std::mem::size_of_val(pixels.as_slice()),
                ),
            )?
        };

        Ok(Self {
            canvas,
            context,

            program,
            _buffer: buffer,
            vertex_array,
            texture,

            last_time: 0f64,

            ortho_matrix: Matrix4::new_identity(),
            perspective_matrix: Matrix4::new_identity(),
            rotation: Degrees(0f32),
        })
    }

    pub fn resize(&mut self) -> Result<()> {
        let window = get_window()?;

        let width = window
            .inner_width()?
            .as_f64()
            .ok_or("expected number for width")?;
        let height = window
            .inner_height()?
            .as_f64()
            .ok_or("expected number for height")?;

        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);

        self.context.viewport(0, 0, width as i32, height as i32);

        self.ortho_matrix =
            Matrix4::new_ortho(0f32, width as f32, height as f32, 0f32, -1f32, 1f32);
        self.perspective_matrix = Matrix4::new_perspective(
            Degrees(90f32).into(),
            width as f32,
            height as f32,
            0.1f32,
            1000f32,
        );

        Ok(())
    }

    pub fn animate(&mut self, time: f64) -> Result<()> {
        let delta = ((time - self.last_time) / 1000f64) as f32;
        self.last_time = time;
        self.rotation = (self.rotation + Degrees(45f32) * Degrees(delta)) % Degrees(360f32);

        self.context.clear_color(0.25, 0.5, 0.75, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.program.use_program();

        // TODO helper for turning matrix into uniform value?
        {
            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.program.get_uniform("uniform_matrix")?.location),
                false,
                &self
                    .perspective_matrix
                    .clone()
                    .append(Matrix4::new_look_at(
                        Vector3::new(self.rotation.cos(), 0f32, self.rotation.sin()) * 6f32
                            + Vector3::new(0f32, 4f32, 0f32),
                        Vector3::new(0f32, 0f32, 0f32),
                        Vector3::new(0f32, 1f32, 0f32),
                    ))
                    .flatten(),
            );
        }

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE1);
        self.texture.bind();
        // TODO helper for sending primitives to uniform?
        self.context.uniform1i(
            Some(&self.program.get_uniform("uniform_texture")?.location),
            1,
        );

        self.vertex_array.bind();
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        self.context.bind_vertex_array(None);

        self.context.use_program(None);

        Ok(())
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    if let Err(e) = main_impl() {
        error!("fatal error: {e:?}");
    }
}

fn main_impl() -> Result<()> {
    let window = get_window()?;
    let body = get_body()?;
    let document = get_document()?;

    while let Some(child) = body.first_child() {
        body.remove_child(&child)?;
    }

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "failed to create canvas")?;
    body.append_child(&canvas)?;
    canvas.style().set_property("position", "absolute")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    canvas.style().set_property("left", "0")?;
    canvas.style().set_property("top", "0")?;

    let context = {
        let options = serde_wasm_bindgen::to_value(&HashMap::from([(
            "powerPreference",
            "high-performance",
        )]))?;
        canvas
            .get_context_with_context_options("webgl2", &options)?
            .ok_or("failed to make webgl2 context")?
            .dyn_into::<WebGl2RenderingContext>()
            .map_err(|_| "expected webgl2 context but got some other kind of context")?
    };

    let state = Rc::new(Mutex::new(State::new(canvas, context)?));

    {
        let mut state = state.lock().unwrap();
        state.resize()?;
    }

    {
        let state = state.clone();

        let closure: Closure<dyn Fn()> = Closure::new(move || {
            let mut state = state.lock().unwrap();
            if let Err(e) = state.resize() {
                error!("error resizing: {e:?}");
            }
        });
        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();

        request_animation_frame_loop(move |_time| Ok(RequestAnimationFrameStatus::Continue))?;
    }

    {
        let state = state.clone();
        request_animation_frame_loop(move |time| {
            let mut state = state.lock().unwrap();
            state.animate(time)?;
            Ok(RequestAnimationFrameStatus::Continue)
        })?;
    }

    Ok(())
}
