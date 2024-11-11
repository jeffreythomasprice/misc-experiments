use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use axum::{
    extract::{
        ws::{self, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::Id;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::spawn,
};
use tracing::*;

#[derive(Clone)]
pub struct ConnectedClients {
    clients: Arc<Mutex<HashMap<Id, Client>>>,
}

impl ConnectedClients {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientDescription {
    addr: SocketAddr,
    user_agent: Option<String>,
    id: Id,
    name: Option<String>,
}

struct Client {
    description: ClientDescription,
    sender: Sender<WebsocketServerToClientMessage>,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<ConnectedClients>,
) -> impl IntoResponse {
    let client = ClientDescription {
        addr,
        user_agent: if let Some(TypedHeader(user_agent)) = user_agent {
            Some(user_agent.to_string())
        } else {
            None
        },
        id: Id::new(),
        name: None,
    };
    info!("ws client connected: {:?}", client);

    ws.on_upgrade(move |socket| handle_socket(state, socket, client))
}

async fn handle_socket(state: ConnectedClients, socket: WebSocket, client_description: ClientDescription) {
    let (sender, mut receiver) =
        websocket_to_json_channels::<WebsocketServerToClientMessage, WebsocketClientToServerMessage>(socket, &client_description).await;

    {
        let state = state.clone();
        let client_description = client_description.clone();
        let sender = sender.clone();
        spawn(async move {
            while let Some(message) = receiver.recv().await {
                trace!(
                    "received websocket message, sender: {:?}, message: {:?}",
                    client_description,
                    message
                );
                match message {
                    WebsocketClientToServerMessage::Hello { name } => {
                        debug!("client updated name, client: {:?}, new name: {}", client_description, name);

                        // update name
                        {
                            let mut clients = state.clients.lock().unwrap();
                            match clients.get_mut(&client_description.id) {
                                Some(client) => client.description.name = Some(name),
                                None => error!(
                                    "expected to be able to update name for client {:?} but no such client found",
                                    client_description
                                ),
                            };
                        }

                        // send back id
                        if let Err(e) = sender
                            .send(WebsocketServerToClientMessage::Welcome {
                                id: client_description.id.clone(),
                            })
                            .await
                        {
                            error!(
                                "error sending welcome message to client, client: {:?}, error: {:?}",
                                client_description, e
                            );
                        }
                    }
                    WebsocketClientToServerMessage::Message { id, timestamp, payload } => {
                        // TODO send to kafka
                    }
                };
            }
        });
    }

    let mut clients = state.clients.lock().unwrap();
    clients.insert(
        client_description.id.clone(),
        Client {
            description: client_description,
            sender,
        },
    );
}

async fn websocket_to_json_channels<S, R>(socket: WebSocket, client_description: &ClientDescription) -> (Sender<S>, Receiver<R>)
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
