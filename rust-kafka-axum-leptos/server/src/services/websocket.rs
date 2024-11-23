use std::net::SocketAddr;

use axum::extract::ws::{self, WebSocket};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use shared::Id;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::spawn,
};
use tracing::*;

#[derive(Debug, Clone)]
pub struct ClientDescription {
    pub addr: SocketAddr,
    pub user_agent: Option<String>,
    pub id: Id,
    pub name: Option<String>,
}

impl ClientDescription {
    pub fn new(addr: SocketAddr, user_agent: Option<TypedHeader<UserAgent>>) -> Self {
        Self {
            addr,
            user_agent: if let Some(TypedHeader(user_agent)) = user_agent {
                Some(user_agent.to_string())
            } else {
                None
            },
            id: Id::new(),
            name: None,
        }
    }
}

pub async fn websocket_to_json_channels<S, R>(socket: WebSocket, client_description: &ClientDescription) -> (Sender<S>, Receiver<R>)
where
    S: Serialize + Send + 'static,
    R: DeserializeOwned + Send + 'static,
{
    let (mut sender, mut receiver) = socket.split();

    let (output_sender, mut output_receiver) = channel::<S>(1);
    {
        let client_description = client_description.clone();
        spawn(async move {
            while let Some(message) = output_receiver.recv().await {
                match serde_json::to_string(&message) {
                    Ok(message) => {
                        if let Err(e) = sender.send(ws::Message::Text(message)).await {
                            error!(
                                "error sending outgoing message to websocket, client: {:?}, error: {:?}",
                                client_description, e
                            );
                        }
                    }
                    Err(e) => error!(
                        "error serializing outgoing message, client {:?}, error: {:?}",
                        client_description, e
                    ),
                };
            }
        });
    }

    let (input_sender, input_receiver) = channel::<R>(1);
    {
        let client_description = client_description.clone();
        spawn(async move {
            while let Some(message) = receiver.next().await {
                match message {
                    Ok(ws::Message::Text(message)) => {
                        match serde_json::from_str(&message) {
                            Ok(message) => {
                                if let Err(e) = input_sender.send(message).await {
                                    error!(
                                        "error sending incoming message to channel for websocket, client: {:?}, error: {:?}",
                                        client_description, e
                                    );
                                }
                            }
                            Err(e) => error!(
                                "error deserializing incoming websocket message, client: {:?}, error: {:?}",
                                client_description, e
                            ),
                        };
                    }
                    Ok(ws::Message::Binary(_)) => {
                        todo!("handle binary messages")
                    }
                    Err(e) => error!("error receiving from websocket {:?}, error: {:?}", client_description, e),
                    _ => (),
                };
            }
        });
    }

    (output_sender, input_receiver)
}
