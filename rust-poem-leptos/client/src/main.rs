use leptos::ev::KeyboardEvent;
use leptos::*;
use log::*;
use shared::models::{ClientHelloRequest, ClientHelloResponse};
use shared::websockets::{Message, WebSocketChannel};
use std::error::Error;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::JsValue;

#[component]
pub fn Login<F>(cx: Scope, submit: F) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    let (value, set_value) = create_signal(cx, "".to_string());

    let on_submit = Rc::new(move || {
        submit(value());
    });

    let on_button_click = {
        let on_submit = on_submit.clone();
        move |_| {
            on_submit();
        }
    };

    let on_input_keyup = move |e: KeyboardEvent| {
        if e.key() == "Enter" {
            on_submit();
        }
    };

    let on_input_input = move |e| {
        set_value(event_target_value(&e));
    };

    view! { cx,
        <div>
            <input type="text" autofocus on:keyup=on_input_keyup on:input=on_input_input />
            <button on:click=on_button_click>Start</button>
        </div>
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, None);

    let login = move |name| {
        debug!("TODO JEFF login: {name}");
        set_name(Some(name));
    };

    let content = move || match name() {
        Some(name) => view! { cx,
            <>
            <div>{name}</div>
            </>
        },
        None => view! { cx,
            <>
            <Login submit=login/>
            </>
        },
    };

    view! { cx,
        <>{content}</>
    }
}

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        if let Err(e) = test_request().await {
            log::error!("error doing test request: {e:?}");
        }
        if let Err(e) = test_websockets().await {
            log::error!("error doing test websockets: {e:?}");
        }
    });

    mount_to_body(|cx| {
        view! { cx,
            <App/>
        }
    })
}

async fn test_request() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8001/client")
        .json(&ClientHelloRequest {
            name: "tester".to_string(),
        })
        .send()
        .await?;
    let response_body: ClientHelloResponse = response.json().await?;
    debug!("response = {response_body:?}");
    Ok(())
}

async fn test_websockets() -> Result<(), JsValue> {
    let (sender, mut receiver) =
        shared::websockets::client::connect("ws://127.0.0.1:8001/ws")?.split();

    spawn_local(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                Ok(Message::Text(value)) => {
                    debug!("TODO JEFF got text message from server, {}", value)
                }
                Ok(Message::Binary(value)) => debug!(
                    "TODO JEFF got binary message from client, {} bytes",
                    value.len()
                ),
                Err(_e) => log::error!("TODO JEFF error from websocket"),
            }
        }
    });

    spawn_local(async move {
        if let Err(e) = sender
            .send(Message::Text(
                "TODO JEFF test message from client".to_string(),
            ))
            .await
        {
            log::error!("TODO JEFF error sending test message: {e:?}");
        }
    });

    Ok(())
}
