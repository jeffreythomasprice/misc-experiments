use std::{
    cell::RefCell,
    mem::forget,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use crate::{
    dom::{body, create_canvas, window},
    geom::Size,
};
use anyhow::{anyhow, Result};
use log::*;
use serde::Serialize;
use web_sys::WebGl2RenderingContext;
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    HtmlCanvasElement,
};

pub enum NextEventHandler {
    NoChange,
    ChangeTo(Box<dyn EventHandler>),
}

pub trait EventHandler {
    fn activate(&mut self, context: &WebGl2RenderingContext) -> Result<()>;
    fn deactivate(&mut self) -> Result<()>;
    fn resize(
        &mut self,
        context: &WebGl2RenderingContext,
        size: Size<u32>,
    ) -> Result<NextEventHandler>;
    fn render(&mut self, context: &WebGl2RenderingContext) -> Result<NextEventHandler>;
    fn update(&mut self, delta: chrono::TimeDelta) -> Result<NextEventHandler>;
}

pub fn run(initial: Box<dyn EventHandler>) -> Result<()> {
    let canvas = Arc::new(create_canvas()?);
    body()?.replace_children_with_node_1(&canvas);

    #[derive(Serialize)]
    struct WebGLOptions {
        #[serde(rename = "powerPreference")]
        power_preference: String,
    }
    let context: Arc<WebGl2RenderingContext> = Arc::new(
        canvas
            .get_context_with_context_options(
                "webgl2",
                &serde_wasm_bindgen::to_value(&WebGLOptions {
                    power_preference: "high-performance".to_owned(),
                })
                .map_err(|e| anyhow!("failed to serialize webgl options: {e:?}"))?,
            )
            .map_err(|e| anyhow!("failed to create webgl context: {e:?}"))?
            .ok_or(anyhow!("failed to create webgl context",))?
            .dyn_into()
            .map_err(|e| {
                anyhow!("created a canvas graphics context, but it wasn't the expected type: {e:?}")
            })?,
    );

    let current_state = Arc::new(Mutex::new(RefCell::new(initial)));
    {
        let mut current_state = current_state.lock().unwrap();
        if let Err(e) = current_state.deref_mut().borrow_mut().activate(&context) {
            error!("error activating initial state: {e:?}");
            return Err(anyhow!("failed to activate initial state"));
        };
    }

    // resize events
    {
        let current_state = current_state.clone();
        let canvas = canvas.clone();
        let context = context.clone();
        let c = Closure::<dyn Fn()>::new(move || {
            resize(current_state.clone(), canvas.clone(), context.clone());
        });
        window()?
            .add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add resize callback to window: {e:?}"))?;
        // don't ever free this so the js callback stays valid
        forget(c);
    }

    // initial resize
    resize(current_state.clone(), canvas.clone(), context.clone());

    //  // mouse down events
    //  {
    // 	let state = state.clone();
    // 	let canvas = state.borrow_mut().canvas.clone();
    // 	let c =
    // 		Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
    // 			let context = state.borrow().context();
    // 			if let Err(e) = state
    // 				.borrow_mut()
    // 				.event_handler
    // 				.borrow_mut()
    // 				.handle_event(Event::MouseDown(MouseEvent { context, event }))
    // 			{
    // 				error!("error handling mouse move: {e:?}");
    // 			}
    // 		});
    // 	canvas
    // 		.add_event_listener_with_callback("mousedown", c.as_ref().unchecked_ref())
    // 		.map_err(|e| Error::Js(e.into()))?;
    // 	// don't ever free this so the js callback stays valid
    // 	forget(c);
    // }

    // // mouse up events
    // {
    // 	let state = state.clone();
    // 	let canvas = state.borrow_mut().canvas.clone();
    // 	let c =
    // 		Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
    // 			let context = state.borrow().context();
    // 			if let Err(e) = state
    // 				.borrow_mut()
    // 				.event_handler
    // 				.borrow_mut()
    // 				.handle_event(Event::MouseUp(MouseEvent { context, event }))
    // 			{
    // 				error!("error handling mouse move: {e:?}");
    // 			}
    // 		});
    // 	canvas
    // 		.add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
    // 		.map_err(|e| Error::Js(e.into()))?;
    // 	// don't ever free this so the js callback stays valid
    // 	forget(c);
    // }

    // // mouse move events
    // {
    // 	let state = state.clone();
    // 	let canvas = state.borrow_mut().canvas.clone();
    // 	let c =
    // 		Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
    // 			let context = state.borrow().context();
    // 			if let Err(e) = state
    // 				.borrow_mut()
    // 				.event_handler
    // 				.borrow_mut()
    // 				.handle_event(Event::MouseMove(MouseMoveEvent { context, event }))
    // 			{
    // 				error!("error handling mouse move: {e:?}");
    // 			}
    // 		});
    // 	canvas
    // 		.add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
    // 		.map_err(|e| Error::Js(e.into()))?;
    // 	// don't ever free this so the js callback stays valid
    // 	forget(c);
    // }

    // // key down events
    // {
    // 	let state = state.clone();
    // 	let canvas = state.borrow_mut().canvas.clone();
    // 	let c = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
    // 		move |event: web_sys::KeyboardEvent| {
    // 			let context = state.borrow().context();
    // 			if let Err(e) = state
    // 				.borrow_mut()
    // 				.event_handler
    // 				.borrow_mut()
    // 				.handle_event(Event::KeyDown(KeyboardEvent { context, event }))
    // 			{
    // 				error!("error handling mouse move: {e:?}");
    // 			}
    // 		},
    // 	);
    // 	window()
    // 		.map_err(|e| Error::Js(e))?
    // 		.add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref())
    // 		.map_err(|e| Error::Js(e.into()))?;
    // 	// don't ever free this so the js callback stays valid
    // 	forget(c);
    // }

    // // key up events
    // {
    // 	let state = state.clone();
    // 	let canvas = state.borrow_mut().canvas.clone();
    // 	let c = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
    // 		move |event: web_sys::KeyboardEvent| {
    // 			let context = state.borrow().context();
    // 			if let Err(e) = state
    // 				.borrow_mut()
    // 				.event_handler
    // 				.borrow_mut()
    // 				.handle_event(Event::KeyUp(KeyboardEvent { context, event }))
    // 			{
    // 				error!("error handling mouse move: {e:?}");
    // 			}
    // 		},
    // 	);
    // 	window()
    // 		.map_err(|e| Error::Js(e))?
    // 		.add_event_listener_with_callback("keyup", c.as_ref().unchecked_ref())
    // 		.map_err(|e| Error::Js(e.into()))?;
    // 	// don't ever free this so the js callback stays valid
    // 	forget(c);
    // }

    // keep track of time, and kick off the first frame
    let last_tick = Arc::new(Mutex::new(chrono::TimeDelta::zero()));
    request_animation_frame(current_state.clone(), last_tick, context.clone());

    // TODO should be async, wait forever until an error occurs in an event handler transition

    Ok(())
}

// resize event
fn resize(
    current_state: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    canvas: Arc<HtmlCanvasElement>,
    context: Arc<WebGl2RenderingContext>,
) {
    update_state(
        current_state.clone(),
        context.clone(),
        |s| {
            let window = window()?;
            let width: f64 = window
                .inner_width()
                .map_err(|e| anyhow!("error getting window width: {e:?}"))?
                .try_into()
                .map_err(|e| {
                    anyhow!("error getting window width, expected f64 but failed to cast: {e:?}")
                })?;
            let width: u32 = width.floor() as u32;
            let height: f64 = window
                .inner_height()
                .map_err(|e| anyhow!("error getting window height: {e:?}"))?
                .try_into()
                .map_err(|e| {
                    anyhow!("error getting window height, expected f64 but failed to cast: {e:?}")
                })?;
            let height: u32 = height.floor() as u32;
            canvas.set_width(width);
            canvas.set_height(height);
            let size = Size { width, height };
            trace!("resize: {size:?}");
            s.resize(&context, size)
        },
        "resize",
    );
}

// render and update events
fn request_animation_frame(
    current_state: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    last_tick: Arc<Mutex<chrono::TimeDelta>>,
    context: Arc<WebGl2RenderingContext>,
) {
    let current_state = current_state.clone();
    let last_tick = last_tick.clone();
    if let Err(e) = (move || -> Result<()> {
        {
            let c = Closure::once_into_js(move |time: f64| {
                {
                    let mut last_tick = last_tick.lock().unwrap();
                    let time = chrono::TimeDelta::milliseconds(time.floor() as i64);
                    let delta = time - *last_tick;
                    *last_tick = time;
                    update_state(
                        current_state.clone(),
                        context.clone(),
                        |s| s.update(delta),
                        "update",
                    );
                }

                update_state(
                    current_state.clone(),
                    context.clone(),
                    |s| s.render(&context),
                    "render",
                );

                request_animation_frame(current_state.clone(), last_tick.clone(), context.clone());
            });
            window()?
                .request_animation_frame(c.as_ref().unchecked_ref())
                .map_err(|e| anyhow!("error invokine request_animation_frame: {e:?}"))?;
        }

        Ok(())
    })() {
        error!("error registering next animation frame callback: {e:?}");
    }
}

fn update_state<F>(
    current_state: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    context: Arc<WebGl2RenderingContext>,
    f: F,
    event_type: &str,
) where
    F: FnOnce(&mut Box<dyn EventHandler>) -> Result<NextEventHandler>,
{
    let mut current_state = current_state.lock().unwrap();
    let next = f(&mut current_state.deref_mut().borrow_mut());
    match next {
        Ok(NextEventHandler::NoChange) => (),
        Ok(NextEventHandler::ChangeTo(mut next)) => {
            if let Err(e) = current_state.deref_mut().borrow_mut().deactivate() {
                error!("error deactivating current state: {e:?}");
                // TODO panic?
                return;
            }
            if let Err(e) = next.activate(&context) {
                error!("error activating next state: {e:?}");
                // TODO panic?
                return;
            }
            current_state.deref_mut().replace(next);
        }
        Err(e) => error!("error handling {event_type}: {e:?}"),
    };
}
