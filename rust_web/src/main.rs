use std::{cell::RefCell, future::Future, rc::Rc};

use async_std::{prelude::*, task};
use gloo_console::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, HtmlElement, Request, RequestInit, Response, Window};

#[async_std::main]
async fn main() {
    run().unwrap()
}

fn run() -> Result<(), JsValue> {
    log!("Hello, World!");

    let window = window()?;
    let document = document()?;
    let body = body()?;

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    canvas.style().set_property("position", "absolute")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    canvas.style().set_property("left", "0")?;
    canvas.style().set_property("top", "0")?;

    while body.has_child_nodes() {
        body.remove_child(&body.first_child().unwrap())?;
    }
    body.append_child(&canvas)?;

    let resize_fn = {
        let window = window.clone();
        let canvas = canvas.clone();
        move || {
            // TODO error handling
            let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
            let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
            canvas.set_width(width);
            canvas.set_height(height);
            log!(format!("resize {width} x {height}"));
        }
    };
    resize_fn();
    let resize_closure = Closure::<dyn Fn()>::new(resize_fn);
    window.add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())?;
    // intentionally leak so the callback works after the function returns
    resize_closure.forget();

    let context = canvas
        .get_context("2d")?
        .ok_or("failed to create canvas context")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let animate_closure = Rc::new(RefCell::<Option<Closure<_>>>::new(None));
    let animate_fn = {
        let window = window.clone();
        let canvas = canvas.clone();
        let context = context.clone();
        let animate_closure = animate_closure.clone();
        move || {
            let gradient = context.create_linear_gradient(
                0f64,
                0f64,
                canvas.width() as f64,
                canvas.height() as f64,
            );
            // TODO error handling
            gradient.add_color_stop(0f32, "red").unwrap();
            gradient.add_color_stop(0.5f32, "green").unwrap();
            gradient.add_color_stop(1f32, "blue").unwrap();
            context.set_fill_style(&gradient);
            context.fill_rect(0f64, 0f64, canvas.width() as f64, canvas.height() as f64);

            // TODO error handling
            window
                .request_animation_frame(
                    animate_closure
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
        }
    };
    {
        let animate_closure = animate_closure.clone();
        *animate_closure.borrow_mut() = Some(Closure::<dyn Fn()>::new(animate_fn));
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

    task::block_on(async {
        log!("in task");
        //TODO error handling
        let result = fetch_string("assets/test.txt").await.unwrap();
        log!("result", result);
        log!("task complete");
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
