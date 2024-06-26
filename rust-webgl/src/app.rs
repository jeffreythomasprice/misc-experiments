use std::{cell::RefCell, fmt::Debug, mem::forget, rc::Rc, time::Duration};

use log::*;
use nalgebra::{Point2, Vector2};
use serde::Serialize;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::errors::JsInteropError;

#[derive(Clone)]
pub struct AppContext {
    pub canvas: HtmlCanvasElement,
    pub gl: Rc<WebGl2RenderingContext>,
}

impl Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContext")
            .field("canvas.width", &self.canvas.width())
            .field("canvas.height", &self.canvas.height())
            .finish()
    }
}

pub struct MouseEvent {
    pub context: AppContext,
    pub event: web_sys::MouseEvent,
}

impl MouseEvent {
    pub fn point(&self) -> Point2<u32> {
        Point2::new(self.event.x() as u32, self.event.y() as u32)
    }
}

impl Debug for MouseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MouseEvent")
            .field("context", &self.context)
            .field("point", &self.point())
            .field("button", &self.event.button())
            .finish()
    }
}

pub struct MouseMoveEvent {
    pub context: AppContext,
    pub event: web_sys::MouseEvent,
}

impl MouseMoveEvent {
    pub fn point(&self) -> Point2<u32> {
        Point2::new(self.event.x() as u32, self.event.y() as u32)
    }

    pub fn delta(&self) -> Vector2<i32> {
        Vector2::new(self.event.movement_x(), self.event.movement_y())
    }
}

impl Debug for MouseMoveEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MouseMoveEvent")
            .field("context", &self.context)
            .field("point", &self.point())
            .field("delta", &self.delta())
            .finish()
    }
}

pub struct KeyboardEvent {
    pub context: AppContext,
    pub event: web_sys::KeyboardEvent,
}

impl Debug for KeyboardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardEvent")
            .field("context", &self.context)
            .field("key_code", &self.event.key_code())
            .finish()
    }
}

pub enum Event {
    Resize {
        context: AppContext,
        width: u32,
        height: u32,
    },
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseMoveEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Render {
        context: AppContext,
    },
    Update {
        context: AppContext,
        delta: Duration,
    },
}

pub trait EventHandler<Error>
where
    Self: Sized,
{
    fn init(gl: Rc<WebGl2RenderingContext>) -> Result<Self, Error>;
    fn handle_event(&mut self, event: Event) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error<EventHandlerError> {
    Js(JsInteropError),
    EventHandler(EventHandlerError),
}

pub struct App<EH> {
    event_handler: Rc<RefCell<EH>>,

    canvas: HtmlCanvasElement,
    gl: Rc<WebGl2RenderingContext>,

    last_ticks: f64,
}

impl<EH> App<EH>
where
    EH: 'static,
{
    pub fn run<EHError>() -> Result<(), Error<EHError>>
    where
        EH: EventHandler<EHError>,
        EHError: Debug,
    {
        let canvas = create_canvas().map_err(|e| Error::Js(e))?;
        body()
            .map_err(|e| Error::Js(e))?
            .replace_children_with_node_1(&canvas);

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
                    })
                    .map_err(|e| Error::Js(e.into()))?,
                )
                .map_err(|e| Error::Js(e.into()))?
                .ok_or(Error::Js(JsInteropError::NotFound(
                    "failed to make webgl context".to_owned(),
                )))?
                .dyn_into()
                .map_err(|_| {
                    Error::Js(JsInteropError::CastError(
                        "created a canvas graphics context, but it wasn't the expected type"
                            .to_owned(),
                    ))
                })?,
        );

        let event_handler = EH::init(gl.clone()).map_err(|e| Error::EventHandler(e))?;

        let state = Rc::new(RefCell::new(App {
            event_handler: Rc::new(RefCell::new(event_handler)),

            canvas,
            gl,

            last_ticks: 0.0,
        }));

        // call once on program start because the resize handler won't call until the window actually changes size otherwise
        if let Err(e) = state.borrow_mut().resize() {
            error!("error resizing: {e:?}");
        }

        // resize events
        {
            let state = state.clone();
            let c = Closure::<dyn Fn()>::new(move || {
                if let Err(e) = state.borrow_mut().resize() {
                    error!("error handling resize: {e:?}");
                }
            });
            window()
                .map_err(|e| Error::Js(e))?
                .add_event_listener_with_callback("resize", c.as_ref().unchecked_ref())
                .map_err(|e| Error::Js(e.into()))?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        // mouse down events
        {
            let state = state.clone();
            let canvas = state.borrow_mut().canvas.clone();
            let c =
                Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
                    let context = state.borrow().context();
                    if let Err(e) = state
                        .borrow_mut()
                        .event_handler
                        .borrow_mut()
                        .handle_event(Event::MouseDown(MouseEvent { context, event }))
                    {
                        error!("error handling mouse move: {e:?}");
                    }
                });
            canvas
                .add_event_listener_with_callback("mousedown", c.as_ref().unchecked_ref())
                .map_err(|e| Error::Js(e.into()))?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        // mouse up events
        {
            let state = state.clone();
            let canvas = state.borrow_mut().canvas.clone();
            let c =
                Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
                    let context = state.borrow().context();
                    if let Err(e) = state
                        .borrow_mut()
                        .event_handler
                        .borrow_mut()
                        .handle_event(Event::MouseUp(MouseEvent { context, event }))
                    {
                        error!("error handling mouse move: {e:?}");
                    }
                });
            canvas
                .add_event_listener_with_callback("mouseup", c.as_ref().unchecked_ref())
                .map_err(|e| Error::Js(e.into()))?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        // mouse move events
        {
            let state = state.clone();
            let canvas = state.borrow_mut().canvas.clone();
            let c =
                Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
                    let context = state.borrow().context();
                    if let Err(e) = state
                        .borrow_mut()
                        .event_handler
                        .borrow_mut()
                        .handle_event(Event::MouseMove(MouseMoveEvent { context, event }))
                    {
                        error!("error handling mouse move: {e:?}");
                    }
                });
            canvas
                .add_event_listener_with_callback("mousemove", c.as_ref().unchecked_ref())
                .map_err(|e| Error::Js(e.into()))?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        // key down events
        {
            let state = state.clone();
            let canvas = state.borrow_mut().canvas.clone();
            let c = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |event: web_sys::KeyboardEvent| {
                    let context = state.borrow().context();
                    if let Err(e) = state
                        .borrow_mut()
                        .event_handler
                        .borrow_mut()
                        .handle_event(Event::KeyDown(KeyboardEvent { context, event }))
                    {
                        error!("error handling mouse move: {e:?}");
                    }
                },
            );
            window()
                .map_err(|e| Error::Js(e))?
                .add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref())
                .map_err(|e| Error::Js(e.into()))?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        // key up events
        {
            let state = state.clone();
            let canvas = state.borrow_mut().canvas.clone();
            let c = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |event: web_sys::KeyboardEvent| {
                    let context = state.borrow().context();
                    if let Err(e) = state
                        .borrow_mut()
                        .event_handler
                        .borrow_mut()
                        .handle_event(Event::KeyUp(KeyboardEvent { context, event }))
                    {
                        error!("error handling mouse move: {e:?}");
                    }
                },
            );
            window()
                .map_err(|e| Error::Js(e))?
                .add_event_listener_with_callback("keyup", c.as_ref().unchecked_ref())
                .map_err(|e| Error::Js(e.into()))?;
            // don't ever free this so the js callback stays valid
            forget(c);
        }

        // render and update events
        {
            fn request_animation_frame<EH, EHError>(state: Rc<RefCell<App<EH>>>)
            where
                EH: EventHandler<EHError> + 'static,
                EHError: Debug,
            {
                let state = state.clone();
                if let Err(e) = (move || -> Result<(), JsInteropError> {
                    {
                        let state = state.clone();
                        let c = Closure::once_into_js(move |time| {
                            if let Err(e) = state.borrow_mut().animate(time) {
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

    fn resize<EHError>(&mut self) -> Result<(), JsInteropError>
    where
        EH: EventHandler<EHError>,
        EHError: Debug,
    {
        let width: f64 = window()?.inner_width()?.try_into()?;
        let height: f64 = window()?.inner_height()?.try_into()?;
        let width = width as u32;
        let height = height as u32;

        self.canvas.set_width(width);
        self.canvas.set_height(height);

        let context = self.context();
        let mut eh = self.event_handler.borrow_mut();
        if let Err(e) = eh.handle_event(Event::Resize {
            context,
            width,
            height,
        }) {
            error!("event handler resize error: {e:?}");
        }

        Ok(())
    }

    fn animate<EHError>(&mut self, time: f64) -> Result<(), JsInteropError>
    where
        EH: EventHandler<EHError>,
        EHError: Debug,
    {
        let delta = std::time::Duration::from_millis((time - self.last_ticks) as u64);
        self.last_ticks = time;

        let context = self.context();
        let mut eh = self.event_handler.borrow_mut();
        if let Err(e) = eh.handle_event(Event::Render { context }) {
            error!("error handling render event: {e:?}");
        }

        let context = self.context();
        if let Err(e) = eh.handle_event(Event::Update { context, delta }) {
            error!("event handling update error: {e:?}");
        }

        Ok(())
    }

    fn context(&self) -> AppContext {
        AppContext {
            canvas: self.canvas.clone(),
            gl: self.gl.clone(),
        }
    }
}

pub fn window() -> Result<web_sys::Window, JsInteropError> {
    web_sys::window().ok_or(JsInteropError::NotFound("failed to get window".to_owned()))
}

pub fn document() -> Result<web_sys::Document, JsInteropError> {
    window()?.document().ok_or(JsInteropError::NotFound(
        "failed to get document".to_owned(),
    ))
}

pub fn body() -> Result<web_sys::HtmlElement, JsInteropError> {
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
