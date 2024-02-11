use std::{cell::RefCell, sync::Arc};

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

    // callbacks have to live while they might be invoked by js
    let open_callback = Arc::new(RefCell::new(None));
    let close_callback = Arc::new(RefCell::new(None));
    let error_callback = Arc::new(RefCell::new(None));
    let message_callback = Arc::new(RefCell::new(None));

    let (outgoing_sender, mut outgoing_receiver) = channel::<String>(1);
    let (incoming_sender, incoming_receiver) = channel::<String>(1);

    open_callback.replace(Some({
        let description = description.clone();
        Closure::<dyn FnMut()>::new(move || {
            trace!("websocket opened, {description}");
        })
    }));
    {
        let c = open_callback.borrow();
        client.set_onopen(Some(c.as_ref().unwrap().as_ref().unchecked_ref()));
    }

    close_callback.replace(Some({
        let open_callback = open_callback.clone();
        let close_callback = close_callback.clone();
        let error_callback = error_callback.clone();
        let message_callback = message_callback.clone();
        let description = description.clone();
        Closure::<dyn FnMut()>::new(move || {
            trace!("websocket closed, {description}");
            // take values out of the callbacks, meaning they get dropped, because js can't possibly invoke them any more
            open_callback.borrow_mut().take();
            close_callback.borrow_mut().take();
            error_callback.borrow_mut().take();
            message_callback.borrow_mut().take();
        })
    }));
    {
        let c = close_callback.borrow();
        client.set_onclose(Some(c.as_ref().unwrap().as_ref().unchecked_ref()));
    }

    error_callback.replace(Some({
        let description = description.clone();
        Closure::<dyn FnMut()>::new(move || {
            trace!("websocket error, {description}");
        })
    }));
    {
        let c = error_callback.borrow();
        client.set_onerror(Some(c.as_ref().unwrap().as_ref().unchecked_ref()));
    }

    message_callback.replace(Some({
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
    }));
    {
        let c = message_callback.borrow();
        client.set_onmessage(Some(c.as_ref().unwrap().as_ref().unchecked_ref()));
    }

    leptos::spawn_local(async move {
        while let Some(msg) = outgoing_receiver.recv().await {
            if let Err(e) = client.send_with_str(&msg) {
                error!("error writing to websocket client: {e:?}");
            }
        }
    });

    Ok((outgoing_sender, incoming_receiver))
}
