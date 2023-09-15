use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct LogInRequest {
    pub client_id: String,
    pub name: String,
}

#[derive(Debug)]
pub enum LogInError {
    AlreadyLoggedIn { client_id: String, name: String },
}

#[derive(Clone)]
pub struct Client {
    base_url: String,
    state: Arc<Mutex<RefCell<Option<State>>>>,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
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
            None => state.replace(State::new(request)),
        };

        Ok(())
    }
}

struct State {
    client_id: String,
    name: String,
}

impl State {
    pub fn new(request: LogInRequest) -> Self {
        // TODO JEFF actually do websocket stuff

        Self {
            client_id: request.client_id,
            name: request.name,
        }
    }
}

// TODO JEFF example, delete me

// async fn test_websockets() -> Result<(), JsValue> {
//     let (sender, mut receiver) =
//         shared::websockets::client::connect("ws://127.0.0.1:8001/ws")?.split();

//     spawn_local(async move {
//         while let Some(message) = receiver.recv().await {
//             match message {
//                 Ok(Message::Text(value)) => {
//                     debug!("TODO JEFF got text message from server, {}", value)
//                 }
//                 Ok(Message::Binary(value)) => debug!(
//                     "TODO JEFF got binary message from client, {} bytes",
//                     value.len()
//                 ),
//                 Err(_e) => log::error!("TODO JEFF error from websocket"),
//             }
//         }
//     });

//     spawn_local(async move {
//         if let Err(e) = sender
//             .send(Message::Text(
//                 "TODO JEFF test message from client".to_string(),
//             ))
//             .await
//         {
//             log::error!("TODO JEFF error sending test message: {e:?}");
//         }
//     });

//     Ok(())
// }
