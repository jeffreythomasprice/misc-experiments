use std::{
    cell::RefCell,
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    dom::{body, create_canvas, document, window},
    geometry::size::Size,
};
use anyhow::{anyhow, Result};
use log::*;
use nalgebra_glm::{I32Vec2, U32Vec2, UVec2};
use serde::Serialize;
use tokio::sync::mpsc::{channel, error::TryRecvError, Sender};
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    HtmlCanvasElement,
};

#[derive(Clone)]
pub struct State {
    canvas: Arc<HtmlCanvasElement>,
    pub context: Arc<WebGl2RenderingContext>,
    key_state: Arc<Mutex<HashMap<String, bool>>>,
}

impl State {
    pub fn new(canvas: Arc<HtmlCanvasElement>, context: Arc<WebGl2RenderingContext>) -> Self {
        Self {
            canvas,
            context,
            key_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_pointer_locked(&self) -> Result<bool> {
        match document()?.pointer_lock_element() {
            Some(canvas) if canvas == ***self.canvas => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn set_pointer_lock(&self, b: bool) -> Result<()> {
        if b {
            self.canvas.request_pointer_lock();
        } else {
            document()?.exit_pointer_lock();
        }
        Ok(())
    }

    pub fn is_key_code_pressed(&self, code: &str) -> bool {
        let key_state = self.key_state.lock().unwrap();
        match key_state.get(code) {
            Some(true) => true,
            _ => false,
        }
    }

    fn key_down(&self, event: &KeyPressEvent) {
        let mut key_state = self.key_state.lock().unwrap();
        key_state.insert(event.code(), true);
    }

    fn key_up(&self, event: &KeyPressEvent) {
        let mut key_state = self.key_state.lock().unwrap();
        key_state.insert(event.code(), false);
    }
}

pub type EventHandlerFactory = Box<dyn FnOnce(State) -> Result<Box<dyn EventHandler>>>;

pub enum NextEventHandler {
    NoChange,
    ChangeTo(EventHandlerFactory),
}

pub struct MousePressEvent {
    event: web_sys::MouseEvent,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(i16),
}

impl MousePressEvent {
    pub fn position(&self) -> U32Vec2 {
        U32Vec2::new(self.event.x() as u32, self.event.y() as u32)
    }

    pub fn button(&self) -> MouseButton {
        match self.event.button() {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            x => MouseButton::Other(x),
        }
    }
}

pub struct MouseMoveEvent {
    event: web_sys::MouseEvent,
}

impl MouseMoveEvent {
    pub fn position(&self) -> U32Vec2 {
        U32Vec2::new(self.event.x() as u32, self.event.y() as u32)
    }

    pub fn delta(&self) -> I32Vec2 {
        I32Vec2::new(self.event.movement_x(), self.event.movement_y())
    }
}

pub struct KeyPressEvent {
    event: web_sys::KeyboardEvent,
}

impl KeyPressEvent {
    pub fn code(&self) -> String {
        self.event.code()
    }
}

pub trait EventHandler {
    fn deactivate(&mut self) -> Result<()> {
        Ok(())
    }

    fn resize(&mut self, size: Size<u32>) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn render(&mut self) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn update(&mut self, delta: Duration) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn mouse_down(&mut self, e: &MousePressEvent) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn mouse_up(&mut self, e: &MousePressEvent) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn mouse_move(&mut self, e: &MouseMoveEvent) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn key_down(&mut self, e: &KeyPressEvent) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }

    fn key_up(&mut self, e: &KeyPressEvent) -> Result<NextEventHandler> {
        Ok(NextEventHandler::NoChange)
    }
}

pub async fn run(initial: EventHandlerFactory) -> Result<()> {
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

    let state = State::new(canvas.clone(), context.clone());

    let current_event_handler = match initial(state.clone()) {
        Ok(result) => Arc::new(Mutex::new(RefCell::new(result))),
        Err(e) => return Err(anyhow!("error creating initial state: {e:?}")),
    };

    let (done_sender, mut done_receiver) = channel(1);

    // initial resize
    resize(
        current_event_handler.clone(),
        canvas.clone(),
        state.clone(),
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
        let current_event_handler = current_event_handler.clone();
        let canvas = canvas.clone();
        let state = state.clone();
        let done_sender = done_sender.clone();
        let c = Closure::<dyn Fn()>::new(move || {
            let current_event_handler = current_event_handler.clone();
            let canvas = canvas.clone();
            let state = state.clone();
            let done_sender = done_sender.clone();
            spawn_local(resize(current_event_handler, canvas, state, done_sender));
        });
        window()?
            .add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add resize event handler: {e:?}"))?;
        c
    };

    // mouse down events
    let mouse_down_event_handler = {
        let current_event_handler = current_event_handler.clone();
        let state = state.clone();
        let done_sender = done_sender.clone();
        let c = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let current_event_handler = current_event_handler.clone();
            let state = state.clone();
            let done_sender = done_sender.clone();
            spawn_local(update_state(
                current_event_handler,
                state,
                |s| s.mouse_down(&MousePressEvent { event }),
                "mouse_down",
                done_sender,
            ));
        });
        canvas
            .add_event_listener_with_callback("mousedown", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add mouse down event handler: {e:?}"))?;
        c
    };

    // mouse up events
    let mouse_up_event_handler = {
        let current_event_handler = current_event_handler.clone();
        let state = state.clone();
        let done_sender = done_sender.clone();
        let c = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let current_event_handler = current_event_handler.clone();
            let state = state.clone();
            let done_sender = done_sender.clone();
            spawn_local(update_state(
                current_event_handler,
                state,
                |s| s.mouse_up(&MousePressEvent { event }),
                "mouse_up",
                done_sender,
            ));
        });
        canvas
            .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add mouse up event handler: {e:?}"))?;
        c
    };

    // mouse move events
    let mouse_move_event_handler = {
        let current_event_handler = current_event_handler.clone();
        let state = state.clone();
        let done_sender = done_sender.clone();
        let c = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let current_event_handler = current_event_handler.clone();
            let state = state.clone();
            let done_sender = done_sender.clone();
            spawn_local(update_state(
                current_event_handler,
                state,
                |s| s.mouse_move(&MouseMoveEvent { event }),
                "mouse_move",
                done_sender,
            ));
        });
        canvas
            .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add mouse move event handler: {e:?}"))?;
        c
    };

    // key down events
    let key_down_event_handler = {
        let current_event_handler = current_event_handler.clone();
        let state = state.clone();
        let done_sender = done_sender.clone();
        let c =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                let current_event_handler = current_event_handler.clone();
                let mut state = state.clone();
                let done_sender = done_sender.clone();
                let event = KeyPressEvent { event };
                state.key_down(&event);
                spawn_local(update_state(
                    current_event_handler,
                    state,
                    move |s| s.key_down(&event),
                    "key_down",
                    done_sender,
                ));
            });
        window()?
            .add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add key down event handler: {e:?}"))?;
        c
    };

    // key up events
    let key_up_event_handler = {
        let current_event_handler = current_event_handler.clone();
        let state = state.clone();
        let done_sender = done_sender.clone();
        let c =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                let current_event_handler = current_event_handler.clone();
                let mut state = state.clone();
                let done_sender = done_sender.clone();
                let event = KeyPressEvent { event };
                state.key_up(&event);
                spawn_local(update_state(
                    current_event_handler,
                    state,
                    move |s| s.key_up(&event),
                    "key_up",
                    done_sender,
                ));
            });
        window()?
            .add_event_listener_with_callback("keyup", c.as_ref().unchecked_ref())
            .map_err(|e| anyhow!("failed to add key up event handler: {e:?}"))?;
        c
    };

    // keep track of time, and kick off the first frame
    let last_tick = Arc::new(Mutex::new(Duration::ZERO));
    request_animation_frame(
        current_event_handler.clone(),
        last_tick,
        state.clone(),
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
    current_event_handler: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    canvas: Arc<HtmlCanvasElement>,
    state: State,
    abort: Sender<Result<()>>,
) {
    update_state(
        current_event_handler.clone(),
        state.clone(),
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
            s.resize(size)
        },
        "resize",
        abort,
    )
    .await
}

// render and update events
async fn request_animation_frame(
    current_event_handler: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    last_tick: Arc<Mutex<Duration>>,
    state: State,
    abort: Sender<Result<()>>,
) {
    let current_event_handler = current_event_handler.clone();
    let last_tick = last_tick.clone();
    if let Err(e) = ({
        let abort = abort.clone();
        move || -> Result<()> {
            {
                let c = Closure::once_into_js(move |time: f64| {
                    let current_event_handler = current_event_handler.clone();
                    let last_tick = last_tick.clone();
                    spawn_local(async move {
                        {
                            let mut last_tick = last_tick.lock().unwrap();
                            let time = Duration::from_millis(time.floor() as u64);
                            let delta = time - *last_tick;
                            *last_tick = time;
                            update_state(
                                current_event_handler.clone(),
                                state.clone(),
                                |s| s.update(delta),
                                "update",
                                abort.clone(),
                            )
                            .await;
                        }

                        update_state(
                            current_event_handler.clone(),
                            state.clone(),
                            |s| s.render(),
                            "render",
                            abort.clone(),
                        )
                        .await;

                        request_animation_frame(
                            current_event_handler.clone(),
                            last_tick.clone(),
                            state.clone(),
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
    current_event_handler: Arc<Mutex<RefCell<Box<dyn EventHandler>>>>,
    state: State,
    f: F,
    event_type: &str,
    abort: Sender<Result<()>>,
) where
    F: FnOnce(&mut Box<dyn EventHandler>) -> Result<NextEventHandler>,
{
    let mut current_event_handler = current_event_handler.lock().unwrap();
    let next = f(&mut current_event_handler.deref_mut().borrow_mut());
    match next {
        Ok(NextEventHandler::NoChange) => (),
        Ok(NextEventHandler::ChangeTo(next)) => {
            if let Err(e) = current_event_handler.deref_mut().borrow_mut().deactivate() {
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
            match next(state.clone()) {
                Ok(next) => current_event_handler.deref_mut().replace(next),
                Err(e) => {
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
            };
        }
        Err(e) => {
            if let Err(e) = abort.send(Err(e)).await {
                error!("error sending previous error to abort channel, event type: {event_type}, error: {e:?}");
            }
        }
    };
}
