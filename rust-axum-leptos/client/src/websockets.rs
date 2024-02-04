use std::fmt::Debug;

use js_sys::{ArrayBuffer, JsString};
use leptos::spawn_local;
use log::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{BinaryType, Blob, MessageEvent, WebSocket};

#[derive(Debug)]
pub enum Error {
    Js(JsValue),
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::Js(value)
    }
}

pub fn connect<Outgoing, Incoming>(
    url: &str,
) -> Result<(Sender<Outgoing>, Receiver<Incoming>), Error>
where
    // TODO shouldn't need Debug
    Outgoing: Serialize + Debug + Clone + 'static,
    Incoming: DeserializeOwned + Debug + Clone + 'static,
{
    let (incoming_send, incoming_receive) = channel::<Incoming>(10);
    let (outgoing_send, outgoing_receive) = channel::<Outgoing>(10);

    let ws = WebSocket::new(url)?;

    // TODO test both binary types
    ws.set_binary_type(BinaryType::Arraybuffer);
    // ws.set_binary_type(BinaryType::Blob);

    let onopen_callback = {
        let ws = ws.clone();
        Closure::<dyn FnMut()>::new(move || {
            debug!("TODO onopen");
            let ws = ws.clone();
            let mut this_outgoing_receive = outgoing_receive.resubscribe();
            spawn_local(async move {
                match this_outgoing_receive.recv().await {
                    Ok(message) => {
                        debug!("TODO handle outgoing message: {message:?}");
                        match serde_json::to_string(&message) {
                            Ok(message) => {
                                if let Err(e) = ws.send_with_str(&message) {
                                    error!("error giving outgoing message to websocket: {e:?}");
                                }
                            }
                            Err(e) => error!("error serializing outgoing message: {e:?}"),
                        }
                    }
                    Err(e) => {
                        error!("error receiving message intended for sending to websocket: {e:?}");
                    }
                }
            });
        })
    };
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    // TODO avoid the forget? free when the channels are dead?
    onopen_callback.forget();

    let onclose_callback = Closure::<dyn FnMut()>::new(move || {
        debug!("TODO oncclose");
    });
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    // TODO avoid the forget? free when the channels are dead?
    onclose_callback.forget();

    let onerror_callback = Closure::<dyn FnMut()>::new(move || {
        debug!("TODO onerror");
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    // TODO avoid the forget? free when the channels are dead?
    onerror_callback.forget();

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(message) = e.data().dyn_into::<ArrayBuffer>() {
            debug!("TODO handle array buffer message: {:?}", message);
        } else if let Ok(message) = e.data().dyn_into::<Blob>() {
            debug!("TODO handle text message: {:?}", message);
        } else if let Ok(message) = e.data().dyn_into::<JsString>() {
            debug!("TODO handle text message: {:?}", message);
            match serde_json::from_str(&message.as_string().unwrap()) {
                Ok(message) => {
                    debug!("TODO handle parsed message: {:?}", message);
                    let incoming_send = incoming_send.clone();
                    if let Err(e) = incoming_send.send(message) {
                        error!("error giving incoming message payload to channel: {e:?}");
                    }
                }
                Err(e) => {
                    error!("error deserializing incoming message payload: {e:?}");
                }
            }
        } else {
            error!("message event, unhandled message type: {:?}", e.data());
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // TODO avoid the forget? free when the channels are dead?
    onmessage_callback.forget();

    Ok((outgoing_send, incoming_receive))
}
