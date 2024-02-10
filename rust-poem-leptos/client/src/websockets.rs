use js_sys::{ArrayBuffer, JsString};
use leptos::spawn_local;
use log::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Blob, MessageEvent};

#[derive(Debug)]
pub enum WebsocketError {
    ConnectError,
}

pub fn websocket(url: &str) -> Result<(Sender<String>, Receiver<String>), WebsocketError> {
    let description = format!("websocket to {url}");

    let client = web_sys::WebSocket::new(url).map_err(|e| {
        error!("error creating websocket client: {e:?}");
        WebsocketError::ConnectError
    })?;

    let (outgoing_sender, mut outgoing_receiver) = channel::<String>(1);
    let (incoming_sender, incoming_receiver) = channel::<String>(1);

    let callback = {
        let description = description.clone();
        Closure::<dyn FnMut()>::new(move || {
            trace!("websocket opened, {description}");
        })
    };
    client.set_onopen(Some(callback.as_ref().unchecked_ref()));
    // TODO clean up callbacks somewhere?
    callback.forget();

    let callback = {
        let description = description.clone();
        Closure::<dyn FnMut()>::new(move || {
            trace!("websocket closed, {description}");
        })
    };
    client.set_onclose(Some(callback.as_ref().unchecked_ref()));
    // TODO clean up callbacks somewhere?
    callback.forget();

    let callback = {
        let description = description.clone();
        Closure::<dyn FnMut()>::new(move || {
            trace!("websocket error, {description}");
        })
    };
    client.set_onerror(Some(callback.as_ref().unchecked_ref()));
    // TODO clean up callbacks somewhere?
    callback.forget();

    let callback = {
        let description = description.clone();
        Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(msg) = e.data().dyn_into::<ArrayBuffer>() {
                debug!("TODO handle array buffers");
            } else if let Ok(msg) = e.data().dyn_into::<Blob>() {
                debug!("TODO handle blobs");
            } else if let Ok(msg) = e.data().dyn_into::<JsString>() {
                let incoming_sender = incoming_sender.clone();
                spawn_local(async move {
                    if let Err(e) = incoming_sender.send(msg.as_string().unwrap()).await {
                        error!("error writing to websocket incoming message channel: {e:?}");
                    }
                });
            } else {
                error!(
                    "unhandled websocket message type, {description}: {:?}",
                    e.data()
                );
            }
        })
    };
    client.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    // TODO clean up callbacks somewhere?
    callback.forget();

    leptos::spawn_local(async move {
        while let Some(msg) = outgoing_receiver.recv().await {
            if let Err(e) = client.send_with_str(&msg) {
                error!("error writing to websocket client: {e:?}");
            }
        }
    });

    Ok((outgoing_sender, incoming_receiver))
}
