#![feature(offset_of)]

use std::{cell::RefCell, collections::HashMap, pin::Pin, rc::Rc};

use js_sys::{Float32Array, Uint8Array};
use log::*;

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    window, HtmlCanvasElement, HtmlElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram,
    WebGlShader, WebGlVertexArrayObject,
};

mod errors;
use errors::*;

struct AttributeInfo {
    name: String,
    size: i32,
    type_: u32,
    location: i32,
}

struct ShaderProgram {
    context: Rc<WebGl2RenderingContext>,
    program: WebGlProgram,
    attributes: HashMap<String, AttributeInfo>,
}

impl ShaderProgram {
    pub fn new(
        context: Rc<WebGl2RenderingContext>,
        vertex_shdaer_source: &str,
        fragment_shader_source: &str,
    ) -> Result<Self> {
        fn create_shader(
            context: &WebGl2RenderingContext,
            type_: u32,
            type_str: &str,
            source: &str,
        ) -> Result<WebGlShader> {
            let result = context
                .create_shader(type_)
                .ok_or_else(|| format!("failed to create shader of type {type_str}"))?;

            context.shader_source(&result, source);
            context.compile_shader(&result);

            if !context
                .get_shader_parameter(&result, WebGl2RenderingContext::COMPILE_STATUS)
                .is_truthy()
            {
                let log = context.get_shader_info_log(&result);
                context.delete_shader(Some(&result));
                Err(format!(
                    "error compiling shader of type {type_str}: {log:?}"
                ))?;
            }

            Ok(result)
        }

        let vertex_shader = create_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            "VERTEX",
            vertex_shdaer_source,
        )?;
        let fragment_shader = match create_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            "FRAGMENT",
            fragment_shader_source,
        ) {
            Ok(result) => result,
            Err(e) => {
                context.delete_shader(Some(&vertex_shader));
                Err(e)?
            }
        };

        let program = context
            .create_program()
            .ok_or(format!("failed to create shader program"))?;
        context.attach_shader(&program, &vertex_shader);
        context.attach_shader(&program, &fragment_shader);
        context.link_program(&program);
        context.delete_shader(Some(&vertex_shader));
        context.delete_shader(Some(&fragment_shader));

        if !context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .is_truthy()
        {
            let log = context.get_program_info_log(&program);
            context.delete_program(Some(&program));
            Err(format!("error linking shader program: {log:?}"))?;
        }

        let active_attributes = context
            .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .ok_or_else(|| "expected number of active attributes, got non-number")?
            as u32;
        let mut attributes = HashMap::new();
        for i in 0..active_attributes {
            let attribute = context.get_active_attrib(&program, i).ok_or_else(|| format!("expected attribute at index {i} because we think there are {active_attributes} attributes"))?;
            let location = context.get_attrib_location(&program, &attribute.name());
            attributes.insert(
                attribute.name(),
                AttributeInfo {
                    name: attribute.name(),
                    size: attribute.size(),
                    type_: attribute.type_(),
                    location,
                },
            );
        }

        Ok(Self {
            context,
            program,
            attributes,
        })
    }

    pub fn get_attribute(&self, name: &str) -> Result<&AttributeInfo> {
        Ok(self
            .attributes
            .get(name)
            .ok_or_else(|| format!("no such attribute: {name}"))?)
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.program));
    }
}

fn buffer_data_with_f32(
    context: &WebGl2RenderingContext,
    target: u32,
    src_data: &[f32],
    usage: u32,
) {
    // js typed array views are unsafe
    // if we do any allocations whlie that view is held we might move that data around in memory, invalidating that view
    unsafe {
        let view = Float32Array::view(&src_data);
        context.buffer_data_with_array_buffer_view(target, &view, usage)
    }
}

fn buffer_data_with_u8(context: &WebGl2RenderingContext, target: u32, src_data: &[u8], usage: u32) {
    // js typed array views are unsafe
    // if we do any allocations whlie that view is held we might move that data around in memory, invalidating that view
    unsafe {
        let view = Uint8Array::view(&src_data);
        context.buffer_data_with_array_buffer_view(target, &view, usage)
    }
}

fn buffer_data_with_typed<T>(
    context: &WebGl2RenderingContext,
    target: u32,
    src_data: &[T],
    usage: u32,
) {
    unsafe {
        buffer_data_with_u8(
            context,
            target,
            core::slice::from_raw_parts(
                src_data.as_ptr() as *const u8,
                src_data.len() * std::mem::size_of::<T>(),
            ),
            usage,
        );
    }
}

struct State {
    canvas: HtmlCanvasElement,
    context: Rc<WebGl2RenderingContext>,

    program: ShaderProgram,
    buffer: WebGlBuffer,
    vertex_array: WebGlVertexArrayObject,
}

#[repr(C)]
#[derive(Debug)]
struct Vector2<T> {
    x: T,
    y: T,
}

impl<T> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Debug)]
struct RGBA<T> {
    r: T,
    g: T,
    b: T,
    a: T,
}

impl<T> RGBA<T> {
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }
}

#[repr(C)]
#[derive(Debug)]
struct Vertex {
    position: Vector2<f32>,
    color: RGBA<f32>,
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
        let color_attribute = program.get_attribute("in_color")?;

        let vertex_data = [
            Vertex {
                position: Vector2::new(-0.5f32, -0.5f32),
                color: RGBA::new(1f32, 1f32, 0f32, 1f32),
            },
            Vertex {
                position: Vector2::new(0.5f32, -0.5f32),
                color: RGBA::new(1f32, 0f32, 1f32, 1f32),
            },
            Vertex {
                position: Vector2::new(0.0f32, 0.5f32),
                color: RGBA::new(0f32, 1f32, 1f32, 1f32),
            },
        ];
        let buffer = context
            .create_buffer()
            .ok_or_else(|| "failed to create buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        buffer_data_with_typed(
            &context,
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertex_data,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        let vertex_array = context
            .create_vertex_array()
            .ok_or_else(|| "failed to create vetex array")?;
        context.bind_vertex_array(Some(&vertex_array));
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        context.enable_vertex_attrib_array(position_attribute.location as u32);
        context.vertex_attrib_pointer_with_i32(
            position_attribute.location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            std::mem::offset_of!(Vertex, position) as i32,
        );
        context.enable_vertex_attrib_array(color_attribute.location as u32);
        context.vertex_attrib_pointer_with_i32(
            color_attribute.location as u32,
            4,
            WebGl2RenderingContext::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            std::mem::offset_of!(Vertex, color) as i32,
        );
        context.bind_vertex_array(None);
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        Ok(Self {
            canvas,
            context,
            program,
            buffer,
            vertex_array,
        })
    }

    pub fn resize(&self) -> Result<()> {
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

        Ok(())
    }

    pub fn animate(&self, _time: f64) -> Result<()> {
        self.context.clear_color(0.25, 0.5, 0.75, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.context.use_program(Some(&self.program.program));
        self.context.bind_vertex_array(Some(&self.vertex_array));
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
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

    while let Some(child) = body.first_child() {
        body.remove_child(&child)?;
    }

    let canvas = get_document()?
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

    let state = Rc::new(State::new(canvas, context)?);

    state.resize()?;

    let onresize = {
        let state = state.clone();
        Closure::<dyn Fn()>::new(move || {
            if let Err(e) = state.resize() {
                error!("error resizing: {e:?}");
            }
        })
    };
    window.set_onresize(Some(onresize.as_ref().unchecked_ref()));
    // leak memory on purpose so the callback lives long enough
    onresize.forget();

    // TODO helper around anim frame
    let onanim1 = Rc::new(RefCell::<Option<Closure<dyn Fn(f64)>>>::new(None));
    let onanim2 = onanim1.clone();
    *onanim1.borrow_mut() = {
        let state = state.clone();
        let f = onanim2.clone();
        Some(Closure::<dyn Fn(f64)>::new(move |time: f64| {
            if let Err(e) = state.animate(time) {
                error!("error animating: {e:?}");
            } else {
                let window = match get_window() {
                    Ok(window) => window,
                    Err(e) => {
                        error!("error getting window to request new animation frame: {e:?}");
                        return;
                    }
                };
                if let Err(e) = window
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                {
                    error!("error requesting new animation frame{e:?}");
                }
            }
        }))
    };
    window.request_animation_frame(onanim1.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;

    Ok(())
}

fn get_window() -> Result<web_sys::Window> {
    Ok(window().ok_or("expected window")?)
}

fn get_document() -> Result<web_sys::Document> {
    Ok(get_window()?.document().ok_or("expected document")?)
}

fn get_body() -> Result<HtmlElement> {
    Ok(get_document()?.body().ok_or("expected body")?)
}
