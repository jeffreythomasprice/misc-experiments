use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{self, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::spawn_local,
};
use tracing::*;

#[derive(Clone)]
pub struct ConnectedClients {
    clients: Arc<Mutex<Vec<Client>>>,
}

impl ConnectedClients {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientDescription {
    addr: SocketAddr,
    user_agent: Option<String>,
}

struct Client {
    description: ClientDescription,
    sender: Sender<shared::WebsocketServerToClientMessage>,
    receiver: Receiver<shared::WebsocketClientToServerMessage>,
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
    };
    info!("ws client connected: {:?}", client);

    ws.on_upgrade(move |socket| handle_socket(socket, state, client))
}

async fn handle_socket(socket: WebSocket, state: ConnectedClients, client_description: ClientDescription) {
    let (mut sender, mut receiver) = socket.split();

    let (input_sender, input_receiver) = channel::<shared::WebsocketClientToServerMessage>(1);
    let (output_sender, mut output_receiver) = channel::<shared::WebsocketServerToClientMessage>(1);

    {
        let client_description = client_description.clone();
        spawn_local(async move {
            while let Some(message) = receiver.next().await {
                match message {
                    Ok(ws::Message::Text(message)) => {
                        match serde_json::from_str::<shared::WebsocketClientToServerMessage>(&message) {
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

    {
        let client_description = client_description.clone();
        spawn_local(async move {
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

    /*
    TODO loop and read from input_receiver
    if it's a hello: send back the id
    if it's a message: just log
    */

    let mut clients = state.clients.lock().unwrap();
    clients.push(Client {
        description: client_description,
        sender: output_sender,
        receiver: input_receiver,
    });
}
