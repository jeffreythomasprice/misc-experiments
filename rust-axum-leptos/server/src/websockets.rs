use std::{fmt::Debug, net::SocketAddr, sync::Mutex};

use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    spawn,
    sync::broadcast::{channel, error::RecvError, Receiver, Sender},
    task::spawn_local,
};
use tracing::*;

pub struct Service<Outgoing> {
    clients: Mutex<Vec<WebSocket<Outgoing>>>,
}

struct WebSocket<Outgoing> {
    pub outgoing_send: Sender<Outgoing>,
}

impl<Outgoing> Service<Outgoing>
where
    // TODO shouldn't need Debug
    Outgoing: Serialize + Debug + Clone + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(Vec::new()),
        }
    }

    pub async fn client_connected<Incoming>(
        &self,
        socket: axum::extract::ws::WebSocket,
    ) -> (Sender<Outgoing>, Receiver<Incoming>)
    where
        // TODO shouldn't need Debug
        Incoming: DeserializeOwned + Debug + Clone + Send + 'static,
    {
        // TODO tracing context

        let (incoming_send, incoming_receive) = channel::<Incoming>(10);
        let (outgoing_send, mut outgoing_receive) = channel::<Outgoing>(10);

        let (mut ws_send, ws_receive) = socket.split();

        spawn(async move {
            _ = ws_receive.for_each(move |message| {
                let incoming_send = incoming_send.clone();
                async move {
                    match message {
                        Ok(message) => {
                            match message {
                                axum::extract::ws::Message::Text(message) => {
                                    debug!("TODO incoming message from websocket: {message}");
                                    match serde_json::from_str(&message) {
                                        Ok(message) => {
                                            debug!("TODO deserialized incoming message from websocket: {message:?}");
                                            if let Err(e) = incoming_send.send(message) {
                                                error!(
                                                "error incoming sending message to channel: {e:?}"
                                            );
                                            }
                                        }
                                        Err(e) => {
                                            error!("error deserializing message from websoccket: {e:?}");
                                        }
                                    }
                                }
                                axum::extract::ws::Message::Binary(message) => {
                                    debug!("TODO handle binary messages: {message:?}")
                                }
                                axum::extract::ws::Message::Ping(_) => (),
                                axum::extract::ws::Message::Pong(_) => (),
                                axum::extract::ws::Message::Close(_) => debug!("websocket closed"),
                            };
                        }
                        Err(e) => {
                            error!("error receiving incoming message from websocket: {e:?}");
                        }
                    }
                }
            });
        });

        spawn(async move {
            let mut done = false;
            while !done {
                match outgoing_receive.recv().await {
                    Ok(message) => {
                        debug!("TODO outgoing message to websocket: {message:?}");
                        match serde_json::to_string(&message) {
                            Ok(message) => {
                                debug!("TODO serialized outgoing message to websocket: {message}");
                                if let Err(e) = ws_send
                                    .send(axum::extract::ws::Message::Text(message))
                                    .await
                                {
                                    error!("error sending outgoing message: {e:?}");
                                }
                            }
                            Err(e) => {
                                error!("error serializing outgoing websocket message: {e:?}");
                            }
                        };
                    }
                    Err(RecvError::Closed) => done = true,
                    Err(e) => {
                        error!("error receiving message from channel to send to websocket: {e:?}");
                    }
                }
            }
        });

        {
            let mut clients = self.clients.lock().unwrap();
            clients.push(WebSocket {
                outgoing_send: outgoing_send.clone(),
            });
        }

        (outgoing_send, incoming_receive)
    }

    pub fn send_to_all(&self, message: &Outgoing) {
        let clients = self.clients.lock().unwrap();
        for client in clients.iter() {
            if let Err(e) = client.outgoing_send.send(message.clone()) {
                error!("error sending: {e:?}");
            }
        }
    }
}
