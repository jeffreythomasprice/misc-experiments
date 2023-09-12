use leptos::{mount_to_body, spawn_local, view};
use log::*;
use shared::models::{ClientHelloRequest, ClientHelloResponse};
use shared::websockets::{Message, WebSocketChannel};
use std::error::Error;
use std::panic;
use wasm_bindgen::JsValue;

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        if let Err(e) = test_request().await {
            error!("error doing test request: {e:?}");
        }
        if let Err(e) = test_websockets().await {
            error!("error doing test websockets: {e:?}");
        }
    });

    mount_to_body(|cx| {
        view! { cx,
            <p>"Hello, world!"</p>
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
                    debug!("TODO JEFF got text message from client, {}", value)
                }
                Ok(Message::Binary(value)) => debug!(
                    "TODO JEFF got binary message from client, {} bytes",
                    value.len()
                ),
                Err(_e) => error!("TODO JEFF error from websocket"),
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
            error!("TODO JEFF error sending test message: {e:?}");
        }
    });

    Ok(())
}
