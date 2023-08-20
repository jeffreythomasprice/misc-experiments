use std::{marker::PhantomData, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{ErrorEvent, MessageEvent};

pub trait EventHandler<MessageType> {
    fn onopen(&self);
    fn onclose(&self);
    fn onerror(&self);
    fn onmessage(&self, message: MessageType);
}

pub struct WebSocket<SenderType, ReceiverType>
where
    SenderType: Serialize,
    ReceiverType: DeserializeOwned,
{
    ws: web_sys::WebSocket,
    phantom: PhantomData<(SenderType, ReceiverType)>,
}

#[derive(Debug)]
pub enum SendError {
    Js(JsValue),
    Serialize(serde_json::Error),
}

impl<SenderType, ReceiverType> WebSocket<SenderType, ReceiverType>
where
    SenderType: Serialize,
    ReceiverType: DeserializeOwned,
{
    pub async fn new(
        url: &str,
        event_handler: impl EventHandler<ReceiverType> + 'static,
    ) -> Result<Self, JsValue> {
        let ws = web_sys::WebSocket::new(url)?;

        let event_handler = Arc::new(event_handler);

        _ = JsFuture::from(js_sys::Promise::new(&mut |resolve, _reject| {
            let onopen = {
                let event_handler = event_handler.clone();
                Closure::<dyn FnMut()>::new(move || {
                    event_handler.onopen();
                    resolve.call0(&JsValue::NULL).unwrap();
                })
            };
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            // forget, to avoid cleaning at the end of the function to js can call this layer
            onopen.forget();

            let onclose = {
                let event_handler = event_handler.clone();
                Closure::<dyn FnMut()>::new(move || {
                    event_handler.onclose();
                })
            };
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            // forget, to avoid cleaning at the end of the function to js can call this layer
            onclose.forget();

            let onerror = {
                let event_handler = event_handler.clone();
                Closure::<dyn FnMut(_)>::new(move |_: ErrorEvent| {
                    event_handler.onerror();
                })
            };
            ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            // forget, to avoid cleaning at the end of the function to js can call this layer
            onerror.forget();

            let onmessage = {
                let _event_handler = event_handler.clone();
                Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                    if let Ok(_buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                        todo!("TODO handle array buffer case");
                    } else if let Ok(_blob) = e.data().dyn_into::<web_sys::Blob>() {
                        todo!("TODO handle blob case");
                    } else if let Ok(_text) = e.data().dyn_into::<js_sys::JsString>() {
                        todo!("TODO handle text case");
                    }
                    // event_handler.onmessage(e);
                })
            };
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            // forget, to avoid cleaning at the end of the function to js can call this layer
            onmessage.forget();
        }))
        .await?;

        Ok(Self {
            ws,
            phantom: PhantomData,
        })
    }

    pub fn close(&self) {
        // TODO implement me
    }

    pub fn send(&self, message: SenderType) -> Result<(), SendError> {
        match serde_json::to_string(&message) {
            Ok(text) => self.ws.send_with_str(&text).map_err(SendError::Js)?,
            Err(e) => Err(SendError::Serialize(e))?,
        };
        Ok(())
    }
}
