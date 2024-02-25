mod errors;
mod shaders;

use std::{cell::RefCell, mem::forget, panic, rc::Rc};

use errors::JsInteropError;
use js_sys::{Math, Uint8Array};
use log::*;
use nalgebra::{Matrix4, Unit, Vector3};
use serde::Serialize;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlVertexArrayObject};

use crate::shaders::ShaderProgram;

#[allow(dead_code)]
struct Vec2 {
    x: f32,
    y: f32,
    z: f32,
}

#[allow(dead_code)]
struct RGBA {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct Vertex {
    position: Vec2,
    color: RGBA,
}

struct AppState {
    canvas: HtmlCanvasElement,
    gl: Rc<WebGl2RenderingContext>,
    shader_program: ShaderProgram,
    vertex_array: WebGlVertexArrayObject,

    last_ticks: f64,

    rotation: f32,
}

impl AppState {
    pub fn go() -> Result<(), JsInteropError> {
        let canvas = create_canvas()?;
        body()?.replace_children_with_node_1(&canvas);

        #[derive(Serialize)]
        struct WebGLOptions {
            #[serde(rename = "powerPreference")]
            power_preference: String,
        }
        let gl: Rc<WebGl2RenderingContext> = Rc::new(
            canvas
                .get_context_with_context_options(
                    "webgl2",
                    &serde_wasm_bindgen::to_value(&WebGLOptions {
                        power_preference: "high-performance".to_owned(),
                    })?,
                )?
                .ok_or(JsInteropError::NotFound(
                    "failed to make webgl context".to_owned(),
                ))?
                .dyn_into()
                .map_err(|_| {
                    JsInteropError::CastError(
                        "created a canvas graphics context, but it wasn't the expected type"
                            .to_owned(),
                    )
                })?,
        );

        let shader_program = ShaderProgram::new(
            gl.clone(),
            include_str!("shader.vert"),
            include_str!("shader.frag"),
        )?;

        let array_buffer = gl.create_buffer().ok_or(JsInteropError::NotFound(
            "failed to create buffer".to_owned(),
        ))?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&array_buffer));
        unsafe {
            let data = [
                Vertex {
                    position: Vec2 {
                        x: -1.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    color: RGBA {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    },
                },
                Vertex {
                    position: Vec2 {
                        x: 1.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    color: RGBA {
                        r: 1.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    },
                },
                Vertex {
                    position: Vec2 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                    color: RGBA {
                        r: 0.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                },
            ];
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &Uint8Array::view_mut_raw(
                    data.as_ptr() as *mut u8,
                    core::mem::size_of::<Vertex>() * data.len(),
                ),
                WebGl2RenderingContext::STATIC_DRAW,
            );
        };
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        let vertex_array = gl.create_vertex_array().ok_or(JsInteropError::NotFound(
            "failed to create vertex array object".to_owned(),
        ))?;
        gl.bind_vertex_array(Some(&vertex_array));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&array_buffer));
        {
            let attr = shader_program
                .get_attribute_by_name("positionAttribute")
                .ok_or(JsInteropError::NotFound(
                    "failed to find attribute".to_owned(),
                ))?;
            gl.vertex_attrib_pointer_with_i32(
                attr.index,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                core::mem::size_of::<Vertex>() as i32,
                core::mem::offset_of!(Vertex, position) as i32,
            );
            gl.enable_vertex_attrib_array(attr.index);
        }
        {
            let attr = shader_program
                .get_attribute_by_name("colorAttribute")
                .ok_or(JsInteropError::NotFound(
                    "failed to find attribute".to_owned(),
                ))?;
            gl.vertex_attrib_pointer_with_i32(
                attr.index,
                4,
                WebGl2RenderingContext::FLOAT,
                false,
                core::mem::size_of::<Vertex>() as i32,
                core::mem::offset_of!(Vertex, color) as i32,
            );
            gl.enable_vertex_attrib_array(attr.index);
        }
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        let state = Rc::new(RefCell::new(AppState {
            canvas,
            gl,
            shader_program,
            vertex_array,

            last_ticks: 0.0,

            rotation: 0.0,
        }));

        // call once on program start because the resize handler won't call until the window actually changes size otherwise
        if let Err(e) = state.borrow_mut().resize() {
            error!("error resizing: {e:?}");
        }

        {
            // register as the window resize handler
            let state = state.clone();
            let c = Closure::<dyn Fn()>::new(move || {
                if let Err(e) = state.borrow_mut().resize() {
                    error!("error resizing: {e:?}");
                }
            });
            window()?.add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        {
            fn request_animation_frame(state: Rc<RefCell<AppState>>) {
                let state = state.clone();
                if let Err(e) = (move || -> Result<(), JsInteropError> {
                    {
                        let state = state.clone();
                        let c = Closure::once_into_js(move |time| {
                            if let Err(e) = state.borrow_mut().anim(time) {
                                error!("error invoking animation frame: {e:?}");
                            }

                            request_animation_frame(state.clone());
                        });
                        window()?.request_animation_frame(c.as_ref().unchecked_ref())?;
                    }

                    Ok(())
                })() {
                    error!("error registering next animation frame callback: {e:?}");
                }
            }

            // kick off the first frame
            request_animation_frame(state.clone());
        }

        Ok(())
    }

    pub fn resize(&mut self) -> Result<(), JsInteropError> {
        let width: f64 = window()?.inner_width()?.try_into()?;
        let height: f64 = window()?.inner_height()?.try_into()?;

        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.gl.viewport(0, 0, width as i32, height as i32);

        Ok(())
    }

    pub fn anim(&mut self, time: f64) -> Result<(), JsInteropError> {
        // cornflower blue, #6495ED
        self.gl.clear_color(0.39, 0.58, 0.93, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader_program.use_program();
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(
                &self
                    .shader_program
                    .get_uniform_by_name("projectionMatrix")
                    .ok_or(JsInteropError::NotFound(
                        "failed to find uniform".to_owned(),
                    ))?
                    .location,
            ),
            false,
            Matrix4::new_perspective(
                (self.canvas.width() as f32) / (self.canvas.height() as f32),
                60.0f32.to_radians(),
                1.0,
                100.0,
            )
            .as_slice(),
        );
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(
                &self
                    .shader_program
                    .get_uniform_by_name("modelViewMatrix")
                    .ok_or(JsInteropError::NotFound(
                        "failed to find uniform".to_owned(),
                    ))?
                    .location,
            ),
            false,
            (Matrix4::new_translation(&Vector3::new(0.0, 0.0, -6.0))
                * Matrix4::from_axis_angle(
                    &Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
                    self.rotation,
                ))
            .as_slice(),
        );

        self.gl.bind_vertex_array(Some(&self.vertex_array));
        self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
        self.gl.bind_vertex_array(None);

        self.gl.use_program(None);

        let delta = std::time::Duration::from_millis((time - self.last_ticks) as u64);
        self.last_ticks = time;
        self.rotation += (delta.as_secs_f32() * 90.0f32.to_radians()) % 360.0f32.to_radians();

        Ok(())
    }
}

fn main() -> Result<(), JsInteropError> {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    if let Err(e) = AppState::go() {
        error!("app init error: {e:?}");
    }

    Ok(())
}

fn window() -> Result<web_sys::Window, JsInteropError> {
    web_sys::window().ok_or(JsInteropError::NotFound("failed to get window".to_owned()))
}

fn document() -> Result<web_sys::Document, JsInteropError> {
    window()?.document().ok_or(JsInteropError::NotFound(
        "failed to get document".to_owned(),
    ))
}

fn body() -> Result<web_sys::HtmlElement, JsInteropError> {
    document()?
        .body()
        .ok_or(JsInteropError::NotFound("failed to get body".to_owned()))
}

fn create_canvas() -> Result<web_sys::HtmlCanvasElement, JsInteropError> {
    let result: web_sys::HtmlCanvasElement = document()?
        .create_element("canvas")?
        .dyn_into()
        .map_err(|_| {
            JsInteropError::CastError(
                "created a canvas element, but it wasn't the expected type".to_owned(),
            )
        })?;

    result.style().set_property("position", "absolute")?;
    result.style().set_property("width", "100%")?;
    result.style().set_property("height", "100%")?;
    result.style().set_property("left", "0px")?;
    result.style().set_property("top", "0px")?;

    Ok(result)
}
