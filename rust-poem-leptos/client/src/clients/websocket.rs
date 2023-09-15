use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use leptos::spawn_local;
use shared::{
    models::{ClientToServerMessage, ServerToClientMessage},
    websockets::{Message, WebSocketChannel},
};
use tokio::sync::mpsc::Sender;
use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub struct LogInRequest {
    pub client_id: String,
    pub name: String,
}

#[derive(Debug)]
pub enum LogInError {
    AlreadyLoggedIn { client_id: String, name: String },
    Unknown(String),
}

#[derive(Debug)]
pub enum SendError {
    NotConnected,
    SerializationError(String),
    FailedToSend(String),
}

#[derive(Clone)]
pub struct Client<F>
where
    F: Fn(ServerToClientMessage),
{
    base_url: String,
    callback: Arc<F>,
    state: Arc<Mutex<RefCell<Option<State>>>>,
}

impl<F> Client<F>
where
    F: Fn(ServerToClientMessage) + 'static,
{
    pub fn new(base_url: &str, callback: F) -> Self {
        Self {
            base_url: base_url.to_string(),
            callback: Arc::new(callback),
            state: Arc::new(Mutex::new(RefCell::new(None))),
        }
    }

    pub fn log_in(&self, request: LogInRequest) -> Result<(), LogInError> {
        let state = self.state.lock().unwrap();
        let state = &mut *state.borrow_mut();

        match state {
            Some(existing) => Err(LogInError::AlreadyLoggedIn {
                client_id: existing.client_id.clone(),
                name: existing.name.clone(),
            })?,
            None => state.replace(
                State::new(&self.base_url, request, self.callback.clone())
                    .map_err(|e| LogInError::Unknown(format!("{e:?}")))?,
            ),
        };

        Ok(())
    }

    pub async fn send(&self, message: &ClientToServerMessage) -> Result<(), SendError> {
        match serde_json::to_string(message) {
            Ok(message) => {
                let state = self.state.lock().unwrap();
                let state = &*state.borrow();
                match state {
                    Some(state) => state
                        .sender
                        .send(Message::Text(message))
                        .await
                        .map_err(|e| SendError::FailedToSend(format!("{e:?}"))),
                    None => Err(SendError::NotConnected),
                }
            }
            Err(e) => Err(SendError::SerializationError(format!("{e:?}"))),
        }
    }
}

struct State {
    client_id: String,
    name: String,
    sender: Sender<Message>,
}

impl State {
    pub fn new<F>(base_url: &str, request: LogInRequest, callback: Arc<F>) -> Result<Self, JsValue>
    where
        F: Fn(ServerToClientMessage) + 'static,
    {
        let (sender, mut receiver) =
            shared::websockets::client::connect(format!("{}/ws", base_url).as_str())?.split();

        // TODO be able to stop this
        spawn_local(async move {
            while let Some(message) = receiver.recv().await {
                let message = match message {
                    Ok(Message::Text(value)) => Some(value),
                    Ok(Message::Binary(value)) => match std::str::from_utf8(&value) {
                        Ok(value) => Some(value.to_string()),
                        Err(e) => {
                            log::error!("error parsing websocket message: {e:?}");
                            None
                        }
                    },
                    Err(_e) => {
                        // TODO try to reconnect? signal error?
                        log::error!("TODO JEFF error from websocket");
                        None
                    }
                };
                let message = if let Some(message) = message {
                    message
                } else {
                    continue;
                };

                let message = match serde_json::from_str::<ServerToClientMessage>(&message) {
                    Ok(message) => message,
                    Err(e) => {
                        log::error!("failed to deserialize message: {e:?}");
                        continue;
                    }
                };

                callback(message);
            }
        });

        Ok(Self {
            client_id: request.client_id,
            name: request.name,
            sender,
        })
    }
}
