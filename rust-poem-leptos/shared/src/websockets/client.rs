#![cfg(feature = "client")]

use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use js_sys::{ArrayBuffer, JsString, Uint8Array};
use log::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Blob, MessageEvent, WebSocket};

use super::{Error, Message};

struct Data {
    ws: WebSocket,

    onopen_closure: Option<Closure<dyn FnMut()>>,
    onclose_closure: Option<Closure<dyn FnMut()>>,
    onmessage_closure: Option<Closure<dyn FnMut(MessageEvent)>>,
    onerror_closure: Option<Closure<dyn FnMut()>>,

    incoming_messages_sender: Sender<Result<Message, Error>>,
    incoming_messages_receiver: Option<Receiver<Result<Message, Error>>>,
    outgoing_messages_sender: Option<Sender<Message>>,
    outgoing_messages_receiver: Option<Receiver<Message>>,
}

#[derive(Clone)]
pub struct WebSocketChannel {
    data: Arc<Mutex<Data>>,
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        format!("{value:?}").into()
    }
}

impl WebSocketChannel {
    pub fn new(ws: WebSocket) -> Self {
        let (incoming_messages_sender, incoming_messages_receiver) = channel(1);
        let (outgoing_messages_sender, mut outgoing_messages_receiver) = channel(1);

        let data = Arc::new(Mutex::new(Data {
            ws,

            onopen_closure: None,
            onclose_closure: None,
            onmessage_closure: None,
            onerror_closure: None,

            incoming_messages_sender,
            incoming_messages_receiver: Some(incoming_messages_receiver),
            outgoing_messages_sender: Some(outgoing_messages_sender),
            outgoing_messages_receiver: Some(outgoing_messages_receiver),
        }));

        let result = Self { data: data.clone() };
        let mut data = data.lock().unwrap();

        let onopen_closure = {
            let result = result.clone();
            Closure::new(move || {
                result.onopen();
            })
        };
        data.ws
            .set_onopen(Some(onopen_closure.as_ref().unchecked_ref()));
        data.onopen_closure.replace(onopen_closure);

        let onclose_closure = {
            let result = result.clone();
            Closure::new(move || {
                result.onclose();
            })
        };
        data.ws
            .set_onclose(Some(onclose_closure.as_ref().unchecked_ref()));
        data.onclose_closure.replace(onclose_closure);

        let onmessage_closure = {
            let result = result.clone();
            Closure::new(move |e: MessageEvent| {
                result.onmessage(e);
            })
        };
        data.ws
            .set_onmessage(Some(onmessage_closure.as_ref().unchecked_ref()));
        data.onmessage_closure.replace(onmessage_closure);
        let onerror_closure = {
            let result = result.clone();
            Closure::new(move || {
                result.onerror();
            })
        };
        data.ws
            .set_onerror(Some(onerror_closure.as_ref().unchecked_ref()));
        data.onerror_closure.replace(onerror_closure);

        result
    }

    fn onopen(&self) {
        debug!("TODO JEFF websocket open");

        let data = self.data.clone();
        spawn_local(async move {
            let mut outgoing_messages_receiver = {
                let mut data = data.lock().unwrap();
                data.outgoing_messages_receiver.take().unwrap()
            };
            while let Some(message) = outgoing_messages_receiver.recv().await {
                let data = data.lock().unwrap();
                if let Err(e) = match message {
                    Message::Text(value) => data.ws.send_with_str(value.as_str()),
                    Message::Binary(value) => data.ws.send_with_u8_array(&value),
                } {
                    error!("error sending to websocket: {e:?}");
                }
            }
        });
    }

    fn onclose(&self) {
        debug!("TODO JEFF websocket closed");
    }

    fn onmessage(&self, e: MessageEvent) {
        debug!("TODO JEFF onmessage: {:?}", e.data());

        let data = self.data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            if let Err(e) = if let Ok(value) = e.data().dyn_into::<ArrayBuffer>() {
                handle_array_buffer(value, &data.incoming_messages_sender).await
            } else if let Ok(value) = e.data().dyn_into::<Blob>() {
                handle_blob(value, &data.incoming_messages_sender).await
            } else if let Ok(value) = e.data().dyn_into::<JsString>() {
                handle_text(value, &data.incoming_messages_sender).await
            } else {
                Err(Error::String(format!(
                    "unrecognized type of data from websocket: {:?}",
                    e.data()
                )))
            } {
                error!("{e:?}");
            }
        });
    }

    fn onerror(&self) {
        debug!("TODO JEFF onerror");

        let data = self.data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            if let Err(e) = data
                .incoming_messages_sender
                .send(Err(Error::Unit(())))
                .await
            {
                error!("error handling incoming messages: {e:?}");
            }
        });
    }
}

impl super::WebSocketChannel for WebSocketChannel {
    fn split(&mut self) -> (Sender<Message>, Receiver<Result<Message, Error>>) {
        let mut data = self.data.lock().unwrap();
        (
            data.outgoing_messages_sender
                .take()
                .expect("cannot split twice"),
            data.incoming_messages_receiver
                .take()
                .expect("cannot split twice"),
        )
    }
}

// TODO JEFF different new?
pub fn connect(url: &str) -> Result<WebSocketChannel, JsValue> {
    trace!("connecting to websocket url={url}");
    Ok(WebSocketChannel::new(WebSocket::new(url)?))
}

async fn handle_array_buffer(
    message: ArrayBuffer,
    sender: &Sender<Result<Message, Error>>,
) -> Result<(), Error> {
    let message = Message::Binary(Uint8Array::new(&message).to_vec());
    let result = sender.send(Ok(message)).await.map_err(|e| {
        format!(
            "error handling incoming websocket message, trying to forward as array buffer: {e:?}"
        )
    })?;
    Ok(result)
}

async fn handle_blob(message: Blob, sender: &Sender<Result<Message, Error>>) -> Result<(), Error> {
    handle_array_buffer(
        JsFuture::from(message.array_buffer())
            .await?
            .dyn_into::<ArrayBuffer>()
            .map_err(|e| {
                format!("failed to get array buffer out of message but it was a blob: {e:?}")
            })?,
        sender,
    )
    .await
}

async fn handle_text(
    message: JsString,
    sender: &Sender<Result<Message, Error>>,
) -> Result<(), Error> {
    let message = Message::Text(message.into());
    let result = sender.send(Ok(message)).await.map_err(|e| {
        format!("error handling incoming websocket message, trying to forward as text: {e:?}")
    })?;
    Ok(result)
}
