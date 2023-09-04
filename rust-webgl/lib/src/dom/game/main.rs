use std::{collections::HashMap, rc::Rc, sync::Mutex, time::Duration};

use log::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, WebGl2RenderingContext};

use crate::{
    dom::{
        anim_frame::{request_animation_frame_loop, RequestAnimationFrameStatus},
        getters::{get_body, get_document, get_window},
    },
    errors::Result,
    glmath::vector2::Vector2,
};

use super::{
    event_listeners::EventListener,
    event_state::{self, EventState},
};

pub fn launch<EventListenerImpl, EventListenerFactory>(
    event_listener_factory: EventListenerFactory,
) -> Result<()>
where
    EventListenerImpl: EventListener + 'static,
    EventListenerFactory: Fn(WebGl2RenderingContext) -> Result<EventListenerImpl>,
{
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

    let state = Rc::new(Mutex::new(event_listener_factory(context.clone())?));

    {
        let state = &mut *state.lock().unwrap();
        resize(state, &canvas).map_err(|e| format!("initial resize failed: {e:?}"))?;
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let closure: Closure<dyn Fn()> = Closure::new(move || {
            let state = &mut *state.lock().unwrap();

            if let Err(e) = resize(state, &canvas) {
                error!("error resizing: {e:?}");
            }
        });
        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    let event_state = Rc::new(Mutex::new(EventState::new()));

    {
        let closure: Closure<dyn Fn(_)> = {
            let state = state.clone();
            let canvas = canvas.clone();
            let document = document.clone();
            let event_state = event_state.clone();
            Closure::new(move |e: MouseEvent| {
                let mut state = state.lock().unwrap();
                let mut event_state = event_state.lock().unwrap();

                let is_pointer_locked = if let Some(lock_element) = document.pointer_lock_element()
                {
                    lock_element.is_equal_node(Some(&canvas))
                } else {
                    false
                };

                let location = Vector2::new(e.client_x(), e.client_y());
                let delta = Vector2::new(e.movement_x(), e.movement_y());

                event_state.mousemove(location, is_pointer_locked);

                if let Err(e) = state.mousemove(&event_state, location, delta, is_pointer_locked) {
                    error!("error on mousemove: {e:?}");
                }
            })
        };
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    {
        let state = state.clone();
        let event_state = event_state.clone();
        let closure: Closure<dyn Fn(_)> = Closure::new(move |e: MouseEvent| {
            let mut state = state.lock().unwrap();
            let mut event_state = event_state.lock().unwrap();

            let button = e.button();
            let location = Vector2::new(e.client_x(), e.client_y());

            event_state.mousedown(button, location);

            if let Err(e) = state.mousedown(&event_state, button, location) {
                error!("error on mousedown: {e:?}");
            }
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    {
        let closure: Closure<dyn Fn(_)> = {
            let state = state.clone();
            let canvas = canvas.clone();
            let document = document.clone();
            let event_state = event_state.clone();
            Closure::new(move |e: MouseEvent| {
                let mut state = state.lock().unwrap();
                let mut event_state = event_state.lock().unwrap();

                // toggle mouse grab on left click
                if e.button() == 0 {
                    // toggle pointer lock
                    if match document.pointer_lock_element() {
                        Some(lock_element) => {
                            if lock_element.is_equal_node(Some(&canvas)) {
                                // we have the lock, we should releases it
                                false
                            } else {
                                // some other element has the lock, we should take over
                                // this should never happen?
                                warn!(
                                    "some other element had the pointer lock that we didn't expect"
                                );
                                true
                            }
                        }
                        None => {
                            // no element has the lock, we should enable
                            true
                        }
                    } {
                        canvas.request_pointer_lock();
                    } else {
                        document.exit_pointer_lock();
                    }
                }

                let button = e.button();
                let location = Vector2::new(e.client_x(), e.client_y());

                event_state.mouseup(button, location);

                if let Err(e) = state.mouseup(&event_state, button, location) {
                    error!("error on mouseup: {e:?}");
                }
            })
        };
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    {
        let state = state.clone();
        let event_state = event_state.clone();
        let closure: Closure<dyn Fn(_)> = Closure::new(move |e: KeyboardEvent| {
            let mut state = state.lock().unwrap();
            let mut event_state = event_state.lock().unwrap();

            let key = e.key();
            let key_code = e.key_code();

            event_state.keydown(key.clone(), key_code);

            if let Err(e) = state.keydown(&event_state, key, key_code) {
                error!("error on keydown: {e:?}");
            }
        });
        window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    {
        let state = state.clone();
        let event_state = event_state.clone();
        let closure: Closure<dyn Fn(_)> = Closure::new(move |e: KeyboardEvent| {
            let mut state = state.lock().unwrap();
            let mut event_state = event_state.lock().unwrap();

            let key = e.key();
            let key_code = e.key_code();

            event_state.keyup(key.clone(), key_code);

            if let Err(e) = state.keyup(&event_state, key, key_code) {
                error!("error on keyup: {e:?}");
            }
        });
        window.add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
        // intentionally leak memory to keep this closure alive forever so js can call it
        closure.forget();
    }

    let last_time = Mutex::new(None);
    {
        let state = state.clone();
        request_animation_frame_loop(move |time| {
            let mut state = state.lock().unwrap();

            let mut last_time = last_time.lock().unwrap();
            if let Some(last_time) = &*last_time {
                let delta = Duration::from_secs_f64((time - last_time) / 1000f64);
                let event_state = event_state.lock().unwrap();
                state.animate(delta, &event_state)?;
            }
            last_time.replace(time);

            state.render()?;

            Ok(RequestAnimationFrameStatus::Continue)
        })?;
    }

    Ok(())
}

fn resize<EventListenerImpl>(
    event_listener: &mut EventListenerImpl,
    canvas: &HtmlCanvasElement,
) -> Result<()>
where
    EventListenerImpl: EventListener,
{
    let window = get_window()?;

    let width = (window
        .inner_width()?
        .as_f64()
        .ok_or("expected number for width")? as i64)
        .try_into()?;
    let height = (window
        .inner_height()?
        .as_f64()
        .ok_or("expected number for height")? as i64)
        .try_into()?;

    canvas.set_width(width);
    canvas.set_height(height);

    event_listener.resize(Vector2::new(width, height))?;

    Ok(())
}
