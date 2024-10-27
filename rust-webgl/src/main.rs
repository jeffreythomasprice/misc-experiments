mod error;
mod events;
mod graphics;
mod math;

use bytemuck::{Pod, Zeroable};
use error::Error;
use events::{KeyPressEvent, MouseButton, MouseMoveEvent, MousePressEvent};
use gloo::{
    events::EventListener,
    render::{request_animation_frame, AnimationFrame},
    utils::{body, document, window},
};
use graphics::{
    array_buffer::ArrayBuffer,
    buffer_usage::BufferUsage,
    element_array_buffer::ElementArrayBuffer,
    shader::{AttributePointer, ShaderProgram, Uniform},
};
use js_sys::wasm_bindgen::JsCast;
use log::*;
use math::camera::Camera;
use nalgebra::Vector2;
use nalgebra_glm::Vec3;
use serde::Serialize;
use std::{collections::HashMap, mem::offset_of, panic, rc::Rc, sync::Mutex, time::Duration};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

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
    position: Vec3,
    color: RGBA,
}

struct State {
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<WebGl2RenderingContext>,

    key_state: Rc<Mutex<HashMap<String, bool>>>,

    shader: ShaderProgram,
    position_attribute: AttributePointer,
    color_attribute: AttributePointer,
    projection_matrix_uniform: Uniform,
    model_view_matrix_uniform: Uniform,

    array_buffer: ArrayBuffer<Vertex>,
    element_aray_buffer: ElementArrayBuffer,

    camera: Camera,
}

impl State {
    pub async fn new(
        canvas: Rc<HtmlCanvasElement>,
        context: Rc<WebGl2RenderingContext>,
    ) -> Result<State, Error> {
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
            3,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, position) as i32,
        );

        let color_attribute = AttributePointer::new::<Vertex>(
            shader
                .get_attribute_by_name("color_attribute")
                .ok_or("failed to find color attribute")?
                .clone(),
            4,
            graphics::shader::AttributePointerType::Float,
            false,
            offset_of!(Vertex, color) as i32,
        );

        let projection_matrix_uniform = shader
            .get_uniform_by_name("projection_matrix_uniform")
            .ok_or("failed to find projection matrix uniform")?
            .clone();

        let model_view_matrix_uniform = shader
            .get_uniform_by_name("model_view_matrix_uniform")
            .ok_or("failed to find model view matrix uniform")?
            .clone();

        let array_buffer = ArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[
                Vertex {
                    position: Vec3::new(-1.0, -1.0, 0.0),
                    color: RGBA {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(1.0, -1.0, 0.0),
                    color: RGBA {
                        red: 0.0,
                        green: 1.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(1.0, 1.0, 0.0),
                    color: RGBA {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
                Vertex {
                    position: Vec3::new(-1.0, 1.0, 0.0),
                    color: RGBA {
                        red: 1.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                },
            ],
        )?;

        let element_aray_buffer = ElementArrayBuffer::new_with_data(
            context.clone(),
            BufferUsage::StaticDraw,
            &[0, 1, 2, 2, 3, 0],
        )?;

        Ok(State {
            canvas,
            context,

            key_state: Rc::new(Mutex::new(HashMap::new())),

            shader,
            position_attribute,
            color_attribute,
            projection_matrix_uniform,
            model_view_matrix_uniform,

            array_buffer,
            element_aray_buffer,

            camera: Camera::new(
                60.0f32.to_radians(),
                Vector2::new(0, 0),
                1.0,
                1000.0,
                Vec3::new(0.0, 0.0, 6.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
        })
    }

    pub fn resize(&mut self, width: f64, height: f64) -> Result<(), Error> {
        let width = width.floor() as u32;
        let height = height.floor() as u32;
        self.context.viewport(0, 0, width as i32, height as i32);
        self.camera.set_screen_size(Vector2::new(width, height));
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Error> {
        self.context
            .clear_color(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.shader.use_program();

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.projection_matrix_uniform.location),
            false,
            self.camera.projection_matrix().as_slice(),
        );
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.model_view_matrix_uniform.location),
            false,
            self.camera.model_view_matrix().as_slice(),
            // rotate_y(self.camera.model_view_matrix(), self.rotation).as_slice(),
        );

        self.array_buffer.bind();
        self.element_aray_buffer.bind();

        self.position_attribute.enable();
        self.color_attribute.enable();

        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.element_aray_buffer.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        self.position_attribute.disable();
        self.color_attribute.disable();

        self.array_buffer.bind_none();
        self.element_aray_buffer.bind_none();

        self.shader.use_none();

        Ok(())
    }

    pub fn update(&mut self, delta: Duration) -> Result<(), Error> {
        let forward =
            if self.is_key_code_pressed("ArrowUp") || self.is_key_code_pressed("KeyW") {
                1.0f32
            } else {
                0.0f32
            } - if self.is_key_code_pressed("ArrowDown") || self.is_key_code_pressed("KeyS") {
                1.0f32
            } else {
                0.0f32
            };
        let right = if self.is_key_code_pressed("ArrowRight") || self.is_key_code_pressed("KeyD") {
            1.0f32
        } else {
            0.0f32
        } - if self.is_key_code_pressed("ArrowLeft") || self.is_key_code_pressed("KeyA")
        {
            1.0f32
        } else {
            0.0f32
        };
        let up = if self.is_key_code_pressed("Space") {
            1.0f32
        } else {
            0.0f32
        } - if self.is_key_code_pressed("ShiftLeft") {
            1.0f32
        } else {
            0.0f32
        };
        self.camera.move_based_on_current_axes(
            forward * 5.0f32 * delta.as_secs_f32(),
            up * 5.0f32 * delta.as_secs_f32(),
            right * 5.0f32 * delta.as_secs_f32(),
        );

        Ok(())
    }

    pub fn mouse_down(&mut self, e: &MousePressEvent) -> Result<(), Error> {
        Ok(())
    }

    pub fn mouse_up(&mut self, e: &MousePressEvent) -> Result<(), Error> {
        if let MouseButton::Left = e.button() {
            self.set_pointer_lock(!self.is_pointer_locked());
        }
        Ok(())
    }

    pub fn mouse_move(&mut self, e: &MouseMoveEvent) -> Result<(), Error> {
        if self.is_pointer_locked() {
            self.camera.turn_based_on_mouse_delta(e.delta());
        }
        Ok(())
    }

    pub fn key_down(&mut self, e: &KeyPressEvent) -> Result<(), Error> {
        let mut key_state = self.key_state.lock().unwrap();
        key_state.insert(e.code(), true);
        Ok(())
    }

    pub fn key_up(&mut self, e: &KeyPressEvent) -> Result<(), Error> {
        let mut key_state = self.key_state.lock().unwrap();
        key_state.insert(e.code(), false);
        Ok(())
    }

    fn is_pointer_locked(&self) -> bool {
        match document().pointer_lock_element() {
            Some(canvas) if canvas == ***self.canvas => true,
            _ => false,
        }
    }

    fn set_pointer_lock(&self, b: bool) {
        if b {
            self.canvas.request_pointer_lock();
        } else {
            document().exit_pointer_lock();
        }
    }

    fn is_key_code_pressed(&self, code: &str) -> bool {
        let key_state = self.key_state.lock().unwrap();
        match key_state.get(code) {
            Some(true) => true,
            _ => false,
        }
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
            match State::new(canvas.clone(), Rc::new(context)).await {
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
        let canvas = canvas.clone();
        let state = state.clone();
        EventListener::new(&canvas, "mousedown", move |event| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Ok(event) = event.clone().dyn_into() {
                    if let Err(e) = state.mouse_down(&MousePressEvent { event }) {
                        error!("error handling mousedown event: {e:?}");
                    }
                } else {
                    error!("error converting event types for mousedown event");
                }
            }
        })
        .forget();
    }

    {
        let canvas = canvas.clone();
        let state = state.clone();
        EventListener::new(&canvas, "mouseup", move |event| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Ok(event) = event.clone().dyn_into() {
                    if let Err(e) = state.mouse_up(&MousePressEvent { event }) {
                        error!("error handling mouseup event: {e:?}");
                    }
                } else {
                    error!("error converting event types for mouseup event");
                }
            }
        })
        .forget();
    }

    {
        let canvas = canvas.clone();
        let state = state.clone();
        EventListener::new(&canvas, "mousemove", move |event| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Ok(event) = event.clone().dyn_into() {
                    if let Err(e) = state.mouse_move(&MouseMoveEvent { event }) {
                        error!("error handling mousemove event: {e:?}");
                    }
                } else {
                    error!("error converting event types for mousemove event");
                }
            }
        })
        .forget();
    }

    {
        let state = state.clone();
        EventListener::new(&window(), "keydown", move |event| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Ok(event) = event.clone().dyn_into() {
                    if let Err(e) = state.key_down(&KeyPressEvent { event }) {
                        error!("error handling keydown event: {e:?}");
                    }
                } else {
                    error!("error converting event types for keydown event");
                }
            }
        })
        .forget();
    }

    {
        let state = state.clone();
        EventListener::new(&window(), "keyup", move |event| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Ok(event) = event.clone().dyn_into() {
                    if let Err(e) = state.key_up(&KeyPressEvent { event }) {
                        error!("error handling keyup event: {e:?}");
                    }
                } else {
                    error!("error converting event types for keyup event");
                }
            }
        })
        .forget();
    }

    {
        let state = state.clone();
        anim_loop(move |delta| {
            let state = &mut *state.lock().unwrap();
            if let Some(state) = state {
                if let Err(e) = state.render() {
                    error!("error rendering: {e:?}");
                }
                if let Err(e) = state.update(delta) {
                    error!("error updating: {e:?}");
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

fn anim_loop<F: Fn(Duration) + 'static>(f: F) {
    fn inner<F: Fn(Duration) + 'static>(
        last_anim: Rc<Mutex<Option<AnimationFrame>>>,
        last_time: Duration,
        f: F,
    ) {
        let callback = {
            let last_anim = last_anim.clone();
            let last_time = last_time.clone();
            move |time: f64| {
                let time = Duration::from_millis(time.floor() as u64);
                let delta = time - last_time;
                f(delta);
                inner(last_anim, time, f);
            }
        };

        {
            let last_anim = &mut *last_anim.lock().unwrap();
            last_anim.replace(request_animation_frame(callback));
        }
    }

    inner(Rc::new(Mutex::new(None)), Duration::ZERO, f);
}
