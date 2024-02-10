use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use poem::{
    web::websocket::{Message, WebSocket, WebSocketStream},
    IntoResponse,
};
use serde::{Deserialize, Serialize};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::*;

#[derive(Clone)]
pub struct WebsocketService {
    clients: Arc<Mutex<Vec<WebsocketClient>>>,
}

impl WebsocketService {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn on_upgrade(
        &self,
        ws: WebSocket,
    ) -> (impl IntoResponse, Sender<String>, Receiver<String>) {
        let clients = self.clients.clone();
        let (outgoing_send, mut outgoing_receive) = channel(1);
        let (incoming_sender, incoming_receiver) = channel::<String>(1);
        let result = ws.on_upgrade(move |socket| async move {
            let client = WebsocketClient::new(socket, incoming_sender);

            {
                let client = client.clone();
                tokio::spawn(async move {
                    while let Some(msg) = outgoing_receive.recv().await {
                        client.send(msg).await;
                    }
                });
            }

            let mut clients = clients.lock().unwrap();
            clients.push(client);
        });
        (result, outgoing_send, incoming_receiver)
    }

    pub async fn broadcast(&self, msg: String) {
        let clients = self.clients.lock().unwrap().clone();
        for client in clients.iter() {
            client.send(msg.clone()).await;
        }
    }
}

#[derive(Clone)]
pub struct WebsocketClient {
    outgoing_sender: Sender<String>,
}

impl WebsocketClient {
    pub fn new(socket: WebSocketStream, incoming_sender: Sender<String>) -> Self {
        let (mut sink, mut stream) = socket.split();

        let (outgoing_sender, mut outgoing_receiver) = channel::<String>(1);

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                match std::str::from_utf8(msg.as_bytes()) {
                    Ok(msg) => {
                        trace!("received websocket message: {msg}");
                        if let Err(e) = incoming_sender.send(msg.to_owned()).await {
                            error!("error sending incoming websocket message to channel: {e:?}");
                        }
                    }
                    Err(e) => {
                        error!("received websocket message, but didn't look like utf8: {e:?}")
                    }
                };
            }
            debug!("websocket disconnected");
        });

        tokio::spawn(async move {
            while let Some(msg) = outgoing_receiver.recv().await {
                if let Err(e) = sink.send(Message::Text(msg)).await {
                    error!("error sending to websocket: {e:?}");
                }
            }
        });

        Self { outgoing_sender }
    }

    pub async fn send(&self, msg: String) {
        if let Err(e) = self.outgoing_sender.send(msg).await {
            error!("error sending to outgoing websocket channel: {e:?}");
        }
    }
}
