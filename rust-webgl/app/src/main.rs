#![feature(offset_of)]

use std::{collections::HashMap, rc::Rc, sync::Mutex, time::Duration};

use lib::{
    dom::{
        anim_frame::{request_animation_frame_loop, RequestAnimationFrameStatus},
        getters::{get_body, get_document, get_window},
    },
    errors::Result,
    glmath::{
        angles::{Degrees, Radians},
        fpscamera::FPSCamera,
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
    position: Vector3<f32>,
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

    last_mouse_location: Option<Vector2<i32>>,

    ortho_matrix: Matrix4<f32>,
    perspective_matrix: Matrix4<f32>,
    camera: FPSCamera<f32>,
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
            &cube(
                Vector3::new(0f32, 0f32, 0f32),
                Vector3::new(2f32, 2f32, 2f32),
                Rgba::new(1f32, 1f32, 1f32, 1f32),
            ),
            WebGl2RenderingContext::STATIC_DRAW,
        )?;

        let vertex_array = VertexArray::new(
            context.clone(),
            &[
                VertexArrayAttribute {
                    shader_attribute: position_attribute,
                    buffer: &buffer,
                    size: 3,
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

        let mut camera = FPSCamera::new(
            Vector3::new(4f32, 3f32, 4f32),
            Vector3::new(0f32, 0f32, 1f32),
            Vector3::new(0f32, 1f32, 0f32),
        );
        camera.look_at(Vector3::new(0f32, 0f32, 0f32));

        Ok(Self {
            canvas,
            context,

            program,
            _buffer: buffer,
            vertex_array,
            texture,

            last_time: 0f64,

            last_mouse_location: None,

            ortho_matrix: Matrix4::new_identity(),
            perspective_matrix: Matrix4::new_identity(),
            camera,
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
        let delta = Duration::from_secs_f64((time - self.last_time) / 1000f64);
        self.last_time = time;

        self.context.clear_color(0.25, 0.5, 0.75, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.context.clear_depth(1.0f32);
        self.context.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.context.depth_func(WebGl2RenderingContext::LEQUAL);

        self.program.use_program();

        self.program
            .get_uniform("uniform_matrix")?
            .set_matrixf(self.perspective_matrix.clone().append(self.camera.matrix()));

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE1);
        self.texture.bind();
        self.program.get_uniform("uniform_texture")?.set1i(1);

        self.vertex_array.bind();
        // TODO should be pulling the number of verticies to draw from the number of vertices or indices
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 36);
        self.context.bind_vertex_array(None);

        self.context.use_program(None);

        self.context.disable(WebGl2RenderingContext::DEPTH_TEST);

        Ok(())
    }

    pub fn mousemove(&mut self, location: Vector2<i32>) -> Result<()> {
        if let Some(last) = self.last_mouse_location {
            let delta = location - last;

            let delta = Vector2::new(delta.x as f32, delta.y as f32) / 5f32;
            self.camera
                .turn(Degrees(delta.y).into(), Degrees(delta.x).into())
        }
        self.last_mouse_location = Some(location);

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

    let state = Rc::new(Mutex::new(State::new(canvas.clone(), context)?));

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
    }

    {
        let state = state.clone();
        let closure: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::MouseEvent| {
            let mut state = state.lock().unwrap();
            if let Err(e) = state.mousemove(Vector2::new(e.client_x(), e.client_y())) {
                error!("error on mousemove: {e:?}");
            }
        });
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    // TODO JEFF mouse down, mouse up
    // TODO JEFF grab cursor on mouse down

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

fn cube(center: Vector3<f32>, size: Vector3<f32>, color: Rgba<f32>) -> [Vertex; 36] {
    let half_size = size / 2f32;
    let min = center - half_size;
    let max = center + half_size;
    [
        // min z face
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // max z face
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // min y face
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // max y face
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // min x face
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // max x face
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
    ]
}