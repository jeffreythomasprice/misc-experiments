use std::{cell::RefCell, rc::Rc, time::Duration};

use async_std::task;
use gloo_console::{error, log};
use rand::{rngs::ThreadRng, Rng};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Document, HtmlCanvasElement, HtmlElement, Request, RequestInit, Response,
    WebGl2RenderingContext, WebGlContextAttributes, Window,
};

#[derive(Debug)]
enum AppError {
    Message(String),
    JsValue(JsValue),
}

impl From<JsValue> for AppError {
    fn from(value: JsValue) -> Self {
        AppError::JsValue(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        AppError::Message(value.into())
    }
}

type AppResult<T> = std::result::Result<T, AppError>;

type AppStateHandle = Rc<RefCell<dyn AppState>>;

trait AppState {
    fn activate(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()>;
    fn deactivate(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()>;
    fn resize(&mut self, gl: &WebGl2RenderingContext, width: i32, height: i32) -> AppResult<()>;
    fn render(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()>;
    fn update(
        &mut self,
        gl: &WebGl2RenderingContext,
        time: Duration,
    ) -> AppResult<Option<AppStateHandle>>;
}

struct DemoState {
    random: Rc<RefCell<ThreadRng>>,
    color: (f32, f32, f32),
    remaining_time: Duration,
}

impl DemoState {
    fn new(random: Rc<RefCell<ThreadRng>>) -> DemoState {
        let x = random.borrow_mut().gen_range(0.0f32..=1.0f32);
        let color = {
            let random = random.clone();
            let mut r = random.borrow_mut();
            (
                r.gen_range(0.0f32..=1.0f32),
                r.gen_range(0.0f32..=1.0f32),
                r.gen_range(0.0f32..=1.0f32),
            )
        };
        let remaining_time = random.borrow_mut().gen_range(3.0..=5.0);
        DemoState {
            random,
            color,
            remaining_time: Duration::from_secs_f64(remaining_time),
        }
    }
}

impl AppState for DemoState {
    fn activate(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()> {
        Ok(())
    }

    fn deactivate(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()> {
        Ok(())
    }

    fn resize(&mut self, gl: &WebGl2RenderingContext, width: i32, height: i32) -> AppResult<()> {
        gl.viewport(0, 0, width, height);
        Ok(())
    }

    fn render(&mut self, gl: &WebGl2RenderingContext) -> AppResult<()> {
        let (red, green, blue) = self.color;
        gl.clear_color(red, green, blue, 1.0f32);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        Ok(())
    }

    fn update(
        &mut self,
        gl: &WebGl2RenderingContext,
        time: Duration,
    ) -> AppResult<Option<AppStateHandle>> {
        self.remaining_time -= time;
        if self.remaining_time > Duration::ZERO {
            Ok(None)
        } else {
            Ok(Some(Rc::new(RefCell::new(DemoState::new(
                self.random.clone(),
            )))))
        }
    }
}

#[async_std::main]
async fn main() {
    if let Err(e) = run() {
        error!(format!("fatal {e:?}"))
    }
}

fn run() -> Result<(), AppError> {
    log!("Hello, World!");

    let window = window()?;
    let document = document()?;
    let body = body()?;

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .or(Err("failed to cast into the right type"))?;

    canvas.style().set_property("position", "absolute")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    canvas.style().set_property("left", "0")?;
    canvas.style().set_property("top", "0")?;

    while body.has_child_nodes() {
        body.remove_child(&body.first_child().unwrap())?;
    }
    body.append_child(&canvas)?;

    let context = canvas
        .get_context_with_context_options(
            "webgl2",
            &WebGlContextAttributes::new()
                .power_preference(web_sys::WebGlPowerPreference::HighPerformance),
        )?
        .ok_or("failed to create canvas context")?
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .or(Err("failed to cast into the right type"))?;

    // two layers of Rc<RefCell<_>>
    // outer layer is because we have to share a mutable reference to the current state in multiple closures
    // inner layer is because each state might create or share references to other states, and so will return ref counted instances when asked for next state
    let current_state: Rc<RefCell<AppStateHandle>> = Rc::new(RefCell::new(Rc::new(RefCell::new(
        DemoState::new(Rc::new(RefCell::new(rand::thread_rng()))),
    ))));
    {
        let current_state = current_state.clone();
        let current_state = (*current_state).borrow_mut();
        current_state.as_ref().borrow_mut().activate(&context)?;
    }
    let resize_fn = {
        fn f(
            window: &Window,
            canvas: &HtmlCanvasElement,
            context: &WebGl2RenderingContext,
            current_state: &Rc<RefCell<AppStateHandle>>,
        ) -> Result<(), AppError> {
            let width = window.inner_width()?.as_f64().ok_or("not a number")? as i32;
            let height = window.inner_height()?.as_f64().ok_or("not a number")? as i32;

            log!(format!("resize {width} x {height}"));

            canvas.set_width(width as u32);
            canvas.set_height(height as u32);

            let current_state = (**current_state).borrow_mut();

            current_state
                .as_ref()
                .borrow_mut()
                .resize(context, width, height)?;

            Ok(())
        }

        let window = window.clone();
        let canvas = canvas.clone();
        let context = context.clone();
        let current_state = current_state.clone();
        move || {
            if let Err(e) = f(&window, &canvas, &context, &current_state) {
                error!(format!("error in resize callback: {e:?}"));
            }
        }
    };
    resize_fn();
    let resize_closure = Closure::<dyn Fn()>::new(resize_fn);
    window.add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())?;
    // intentionally leak so the callback works after the function returns
    resize_closure.forget();

    let animate_closure = Rc::new(RefCell::<Option<Closure<_>>>::new(None));
    let animate_fn = {
        fn f(
            context: &WebGl2RenderingContext,
            current_state: &Rc<RefCell<AppStateHandle>>,
            time: JsValue,
            last_time: &Rc<RefCell<Option<f64>>>,
        ) -> Result<(), AppError> {
            let mut current_state = (*current_state).borrow_mut();
            current_state.as_ref().borrow_mut().render(&context)?;

            let now = time.as_f64().ok_or("not a number")?;
            if let Some(last) = *last_time.borrow() {
                let new_state = current_state
                    .as_ref()
                    .borrow_mut()
                    .update(&context, Duration::from_secs_f64((now - last) / 1000f64))?;
                if let Some(new_state) = new_state {
                    log!("TODO JEFF going to new state");
                    new_state.as_ref().borrow_mut().activate(&context)?;
                    current_state.as_ref().borrow_mut().deactivate(&context)?;
                    *current_state = new_state.clone();
                }
            }
            last_time.replace(Some(now));

            Ok(())
        }

        let window = window.clone();
        let context = context.clone();
        let animate_closure = animate_closure.clone();
        let current_state = current_state.clone();
        let last_time = Rc::new(RefCell::new(None));
        move |time: JsValue| {
            if let Err(e) = f(&context, &current_state, time, &last_time) {
                error!(format!("error doing animate: {e:?}"));
            }

            if let Err(e) = window.request_animation_frame(
                animate_closure
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            ) {
                error!(format!("error scheduling next animation: {e:?}"));
            }
        }
    };
    {
        let animate_closure = animate_closure.clone();
        animate_closure.replace(Some(Closure::<dyn Fn(JsValue)>::new(animate_fn)));
    }
    {
        let animate_closure = animate_closure.clone();
        window.request_animation_frame(
            animate_closure
                .borrow()
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )?;
    }

    async fn load_stuff() -> Result<(), AppError> {
        log!("in task");
        let result = fetch_string("assets/test.txt").await?;
        log!("result", result);
        log!("task complete");
        Ok(())
    }
    task::block_on(async {
        if let Err(e) = load_stuff().await {
            error!(format!("error loading resources: {e:?}"))
        }
    });

    Ok(())
}

async fn fetch_string(url: &str) -> Result<JsValue, JsValue> {
    let request = Request::new_with_str_and_init(
        url,
        RequestInit::new()
            .method("GET")
            .mode(web_sys::RequestMode::Cors),
    )?;
    let response = JsFuture::from(window()?.fetch_with_request(&request))
        .await?
        .dyn_into::<Response>()?;
    let response_body = JsFuture::from(response.text()?).await?;
    Ok(response_body)
}

fn window() -> Result<Window, &'static str> {
    web_sys::window().ok_or("failed to get window")
}

fn document() -> Result<Document, &'static str> {
    window()?.document().ok_or("failed to get window")
}

fn body() -> Result<HtmlElement, &'static str> {
    document()?.body().ok_or("failed to get body")
}
