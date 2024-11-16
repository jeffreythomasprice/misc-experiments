use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{ws::WebSocket, ConnectInfo, State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use shared::Id;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use tokio::{sync::mpsc::Sender, task::spawn};
use tracing::*;

use crate::{
    controllers::kafka::Message,
    services::websocket::{websocket_to_json_channels, ClientDescription},
};

use super::kafka::Kafka;

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

struct Client {
    description: ClientDescription,
    sender: Sender<WebsocketServerToClientMessage>,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(connected_clients): State<ConnectedClients>,
    State(kafka): State<Kafka>,
) -> impl IntoResponse {
    let client_description = ClientDescription::new(addr, user_agent);
    info!("ws client connected: {:?}", client_description);

    ws.on_upgrade(move |socket| handle_socket(connected_clients, kafka, socket, client_description))
}

async fn handle_socket(connected_clients: ConnectedClients, kafka: Kafka, socket: WebSocket, client_description: ClientDescription) {
    let (sender, mut receiver) =
        websocket_to_json_channels::<WebsocketServerToClientMessage, WebsocketClientToServerMessage>(socket, &client_description).await;

    {
        let state = connected_clients.clone();
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
                        // send to kafka
                        if let Err(e) = kafka
                            .send_message(Message {
                                id,
                                timestamp,
                                sender: client_description.id.clone(),
                                payload,
                            })
                            .await
                        {
                            error!("error sending message to kafka, error: {:?}", e);
                        }
                    }
                };
            }
        });
    }

    let mut clients = connected_clients.clients.lock().unwrap();
    clients.insert(
        client_description.id.clone(),
        Client {
            description: client_description,
            sender,
        },
    );
}
