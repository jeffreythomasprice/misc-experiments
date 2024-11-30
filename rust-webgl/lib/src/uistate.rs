use crate::error::Error;
use crate::events::{KeyPressEvent, MouseMoveEvent, MousePressEvent};
use crate::math::size::Size;
use gloo::{
    events::EventListener,
    render::{request_animation_frame, AnimationFrame},
    utils::{body, document, window},
};
use js_sys::wasm_bindgen::JsCast;
use log::*;
use serde::Serialize;
use std::{future::Future, panic, rc::Rc, sync::Mutex, time::Duration};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

pub trait UIState {
    fn resize(&mut self, #[allow(unused)] size: Size<u32>) -> Result<(), Error> {
        Ok(())
    }

    fn render(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn update(&mut self, #[allow(unused)] delta: Duration) -> Result<(), Error> {
        Ok(())
    }

    fn mouse_down(&mut self, #[allow(unused)] e: &MousePressEvent) -> Result<(), Error> {
        Ok(())
    }

    fn mouse_up(&mut self, #[allow(unused)] e: &MousePressEvent) -> Result<(), Error> {
        Ok(())
    }

    fn mouse_move(&mut self, #[allow(unused)] e: &MouseMoveEvent) -> Result<(), Error> {
        Ok(())
    }

    fn key_down(&mut self, #[allow(unused)] e: &KeyPressEvent) -> Result<(), Error> {
        Ok(())
    }

    fn key_up(&mut self, #[allow(unused)] e: &KeyPressEvent) -> Result<(), Error> {
        Ok(())
    }
}

pub fn run<R, F>(initial_state_factory: F) -> Result<(), Error>
where
    R: Future<Output = Result<Box<dyn UIState>, Error>>,
    F: FnOnce(Rc<HtmlCanvasElement>, Rc<WebGl2RenderingContext>) -> R + 'static,
{
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
        .map_err(|e| format!("created a canvas graphics context, but it wasn't the expected type: {e:?}"))?;

    let canvas = Rc::new(canvas);
    let state: Rc<Mutex<Option<Box<dyn UIState>>>> = Rc::new(Mutex::new(None));

    {
        let canvas = canvas.clone();
        let state = state.clone();
        spawn_local(async move {
            match initial_state_factory(canvas.clone(), Rc::new(context)).await {
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

fn resize(canvas: &HtmlCanvasElement, state: &mut Box<dyn UIState>) -> Result<(), Error> {
    let width = window().inner_width()?.as_f64().ok_or("expected float")?;
    let height = window().inner_height()?.as_f64().ok_or("expected float")?;
    canvas.set_width(width.floor() as u32);
    canvas.set_height(height.floor() as u32);
    state.resize(Size {
        width: width as u32,
        height: height as u32,
    })
}

fn anim_loop<F: Fn(Duration) + 'static>(f: F) {
    fn inner<F: Fn(Duration) + 'static>(last_anim: Rc<Mutex<Option<AnimationFrame>>>, last_time: Duration, f: F) {
        let callback = {
            let last_anim = last_anim.clone();
            let last_time = last_time;
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
