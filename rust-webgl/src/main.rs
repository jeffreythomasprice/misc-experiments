use std::{cell::RefCell, collections::HashMap, rc::Rc};

use js_sys::Float32Array;
use log::*;

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    window, HtmlCanvasElement, HtmlElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram,
    WebGlShader, WebGlVertexArrayObject,
};

mod errors;
use errors::*;

fn create_shader_program(
    context: &WebGl2RenderingContext,
    vertex_shdaer_source: &str,
    fragment_shader_source: &str,
) -> Result<WebGlProgram> {
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
        context,
        WebGl2RenderingContext::VERTEX_SHADER,
        "VERTEX",
        vertex_shdaer_source,
    )?;
    let fragment_shader = match create_shader(
        context,
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

    let result = context
        .create_program()
        .ok_or(format!("failed to create shader program"))?;
    context.attach_shader(&result, &vertex_shader);
    context.attach_shader(&result, &fragment_shader);
    context.link_program(&result);
    context.delete_shader(Some(&vertex_shader));
    context.delete_shader(Some(&fragment_shader));

    if !context
        .get_program_parameter(&result, WebGl2RenderingContext::LINK_STATUS)
        .is_truthy()
    {
        let log = context.get_program_info_log(&result);
        context.delete_program(Some(&result));
        Err(format!("error linking shader program: {log:?}"))?;
    }

    Ok(result)
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

struct State {
    canvas: HtmlCanvasElement,
    context: WebGl2RenderingContext,

    program: WebGlProgram,
    buffer: WebGlBuffer,
    vertex_array: WebGlVertexArrayObject,
}

impl State {
    pub fn new(canvas: HtmlCanvasElement, context: WebGl2RenderingContext) -> Result<Self> {
        let program = create_shader_program(
            &context,
            include_str!("shaders/shader.vertex"),
            include_str!("shaders/shader.fragment"),
        )?;

        let vertex_data = [-0.5f32, -0.5f32, 0.5f32, -0.5f32, 0.0f32, 0.5f32];
        let buffer = context
            .create_buffer()
            .ok_or_else(|| "failed to create buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        buffer_data_with_f32(
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
        // TODO should look up attribute index by name in the shader
        context.enable_vertex_attrib_array(0);
        context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
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

        self.context.use_program(Some(&self.program));
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
