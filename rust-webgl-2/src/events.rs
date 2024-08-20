use std::{
    cell::RefCell,
    mem::forget,
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    dom::{body, create_canvas, window},
    geom::Size,
};
use anyhow::{anyhow, Result};
use log::*;
use serde::Serialize;
use tokio::sync::mpsc::{channel, error::TryRecvError, Sender};
use wasm_bindgen_futures::spawn_local;
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
    fn update(&mut self, delta: Duration) -> Result<NextEventHandler>;
}

pub async fn run(initial: Box<dyn EventHandler>) -> Result<()> {
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
            return Err(anyhow!("error activating initial state: {e:?}"));
        };
    }

    let (done_sender, mut done_receiver) = channel(1);

    // initial resize
    resize(
        current_state.clone(),
        canvas.clone(),
        context.clone(),
        done_sender.clone(),
    )
    .await;
    // exit early if done_receiver has something?
    match done_receiver.try_recv() {
        Ok(result) => {
            return result;
        }
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => {
            return Err(anyhow!(
                "failed to check error status, receiver is disconnected"
            ));
        }
    };

    // resize events
    let resize_event_handler = {
        let current_state = current_state.clone();
        let canvas = canvas.clone();
        let context = context.clone();
        let done_sender = done_sender.clone();
        let c = Closure::<dyn Fn()>::new(move || {
            let current_state = current_state.clone();
            let canvas = canvas.clone();
            let context = context.clone();
            let done_sender = done_sender.clone();
            spawn_local(resize(current_state, canvas, context, done_sender));
        });
        window()?
            .add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add resize event handler: {e:?}"))?;
        c
    };

    // mouse down events
    let mouse_down_event_handler = {
        let current_state = current_state.clone();
        let canvas = canvas.clone();
        let c = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            // TODO mouse down event
            // let context = state.borrow().context();
            // if let Err(e) = state
            //     .borrow_mut()
            //     .event_handler
            //     .borrow_mut()
            //     .handle_event(Event::MouseDown(MouseEvent { context, event }))
            // {
            //     error!("error handling mouse move: {e:?}");
            // }
        });
        canvas
            .add_event_listener_with_callback("mousedown", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add mouse down event handler: {e:?}"))?;
        c
    };

    // mouse up events
    let mouse_up_event_handler = {
        let current_state = current_state.clone();
        let canvas = canvas.clone();
        let c = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            // TODO mouse up event
            // let context = state.borrow().context();
            // if let Err(e) = state
            //     .borrow_mut()
            //     .event_handler
            //     .borrow_mut()
            //     .handle_event(Event::MouseUp(MouseEvent { context, event }))
            // {
            //     error!("error handling mouse move: {e:?}");
            // }
        });
        canvas
            .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add mouse up event handler: {e:?}"))?;
        c
    };

    // mouse move events
    let mouse_move_event_handler = {
        let current_state = current_state.clone();
        let canvas = canvas.clone();
        let c = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            // TODO mouse move event
            // let context = state.borrow().context();
            // if let Err(e) = state
            //     .borrow_mut()
            //     .event_handler
            //     .borrow_mut()
            //     .handle_event(Event::MouseMove(MouseMoveEvent { context, event }))
            // {
            //     error!("error handling mouse move: {e:?}");
            // }
        });
        canvas
            .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add mouse move event handler: {e:?}"))?;
        c
    };

    // key down events
    let key_down_event_handler = {
        let current_state = current_state.clone();
        let c =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                // TODO key down event
                // let context = state.borrow().context();
                // if let Err(e) = state
                //     .borrow_mut()
                //     .event_handler
                //     .borrow_mut()
                //     .handle_event(Event::KeyDown(KeyboardEvent { context, event }))
                // {
                //     error!("error handling mouse move: {e:?}");
                // }
            });
        window()?
            .add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add key down event handler: {e:?}"))?;
        c
    };

    // key up events
    let key_up_event_handler = {
        let current_state = current_state.clone();
        let c =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                // TODO key up event
                // let context = state.borrow().context();
                // if let Err(e) = state
                //     .borrow_mut()
                //     .event_handler
                //     .borrow_mut()
                //     .handle_event(Event::KeyUp(KeyboardEvent { context, event }))
                // {
                //     error!("error handling mouse move: {e:?}");
                // }
            });
        window()?
            .add_event_listener_with_callback("keyup", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add key up event handler: {e:?}"))?;
        c
    };

    // keep track of time, and kick off the first frame
    let last_tick = Arc::new(Mutex::new(Duration::ZERO));
    request_animation_frame(
        current_state.clone(),
        last_tick,
        context.clone(),
        done_sender,
    )
    .await;

    // wait around forever until something aborts the state machine
    let result = match done_receiver.recv().await {
        Some(result) => result,
        None => Ok(()),
    };

    // detach all event handlers
    if let Err(e) = window()?.remove_event_listener_with_callback(
        "resize",
        resize_event_handler.as_ref().unchecked_ref(),
    ) {
        error!("failed to remove resize event handler: {e:?}");
    }
    if let Err(e) = canvas.remove_event_listener_with_callback(
        "mousedown",
        mouse_down_event_handler.as_ref().unchecked_ref(),
    ) {
        error!("failed to remove mouse down event handler: {e:?}");
    }
    if let Err(e) = canvas.remove_event_listener_with_callback(
        "mouseup",
        mouse_up_event_handler.as_ref().unchecked_ref(),
    ) {
        error!("failed to remove mouse up event handler: {e:?}");
    }
    if let Err(e) = canvas.remove_event_listener_with_callback(
        "mousemove",
        mouse_move_event_handler.as_ref().unchecked_ref(),
    ) {
        error!("failed to remove mouse move event handler: {e:?}");
    }
    if let Err(e) = window()?.remove_event_listener_with_callback(
        "keydown",
        key_down_event_handler.as_ref().unchecked_ref(),
    ) {
        error!("failed to remove key down event handler: {e:?}");
    }
    if let Err(e) = window()?
        .remove_event_listener_with_callback("keyup", key_up_event_handler.as_ref().unchecked_ref())
    {
        error!("failed to remove key up event handler: {e:?}");
    }

    result
}

// resize event
async fn resize(
    current_state: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    canvas: Arc<HtmlCanvasElement>,
    context: Arc<WebGl2RenderingContext>,
    abort: Sender<Result<()>>,
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
            s.resize(&context, size)
        },
        "resize",
        abort,
    )
    .await
}

// render and update events
async fn request_animation_frame(
    current_state: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    last_tick: Arc<Mutex<Duration>>,
    context: Arc<WebGl2RenderingContext>,
    abort: Sender<Result<()>>,
) {
    let current_state = current_state.clone();
    let last_tick = last_tick.clone();
    if let Err(e) = ({
        let abort = abort.clone();
        move || -> Result<()> {
            {
                let c = Closure::once_into_js(move |time: f64| {
                    let current_state = current_state.clone();
                    let last_tick = last_tick.clone();
                    spawn_local(async move {
                        {
                            let mut last_tick = last_tick.lock().unwrap();
                            let time = Duration::from_millis(time.floor() as u64);
                            let delta = time - *last_tick;
                            *last_tick = time;
                            update_state(
                                current_state.clone(),
                                context.clone(),
                                |s| s.update(delta),
                                "update",
                                abort.clone(),
                            )
                            .await;
                        }

                        update_state(
                            current_state.clone(),
                            context.clone(),
                            |s| s.render(&context),
                            "render",
                            abort.clone(),
                        )
                        .await;

                        request_animation_frame(
                            current_state.clone(),
                            last_tick.clone(),
                            context.clone(),
                            abort.clone(),
                        )
                        .await;
                    });
                });
                window()?
                    .request_animation_frame(c.as_ref().unchecked_ref())
                    .map_err(|e| anyhow!("error invokine request_animation_frame: {e:?}"))?;
            }

            Ok(())
        }
    })() {
        if let Err(e) = abort
            .send(Err(anyhow!(
                "error registering next animation frame callback: {e:?}"
            )))
            .await
        {
            error!("error sending error to abort channel, previous error was about failing to register animation frame callback, error: {e:?}");
        }
    }
}

async fn update_state<F>(
    current_state: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    context: Arc<WebGl2RenderingContext>,
    f: F,
    event_type: &str,
    abort: Sender<Result<()>>,
) where
    F: FnOnce(&mut Box<dyn EventHandler>) -> Result<NextEventHandler>,
{
    let mut current_state = current_state.lock().unwrap();
    let next = f(&mut current_state.deref_mut().borrow_mut());
    match next {
        Ok(NextEventHandler::NoChange) => (),
        Ok(NextEventHandler::ChangeTo(mut next)) => {
            if let Err(e) = current_state.deref_mut().borrow_mut().deactivate() {
                if let Err(e) = abort
                    .send(Err(anyhow!(
                        "error deactivating current state, event type: {event_type}, error: {e:?}"
                    )))
                    .await
                {
                    error!("error sending previous error to abort channel, event type: {event_type}, error: {e:?}");
                }
                return;
            }
            if let Err(e) = next.activate(&context) {
                if let Err(e) = abort
                    .send(Err(anyhow!(
                        "error activating next state, event type: {event_type}, error: {e:?}"
                    )))
                    .await
                {
                    error!("error sending previous error to abort channel, event type: {event_type}, error: {e:?}");
                }
                return;
            }
            current_state.deref_mut().replace(next);
        }
        Err(e) => {
            if let Err(e) = abort.send(Err(e)).await {
                error!("error sending previous error to abort channel, event type: {event_type}, error: {e:?}");
            }
        }
    };
}
