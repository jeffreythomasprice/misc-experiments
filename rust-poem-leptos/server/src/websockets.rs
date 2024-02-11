use futures_util::{SinkExt, StreamExt};
use poem::{
    web::websocket::{WebSocket, WebSocketStream},
    IntoResponse,
};
use serde::{de::DeserializeOwned, Serialize};
use shared::websockets::Message;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::*;

#[derive(Clone)]
pub struct Service<SendT> {
    clients: Arc<Mutex<Vec<Sender<SendT>>>>,
}

impl<SendT> Service<SendT>
where
    SendT: Serialize + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn on_upgrade<ReceiveT>(
        &self,
        ws: WebSocket,
    ) -> (impl IntoResponse, Sender<SendT>, Receiver<ReceiveT>)
    where
        ReceiveT: DeserializeOwned + Send + 'static,
    {
        let clients = self.clients.clone();
        let (outgoing_send, mut outgoing_receive) = channel::<SendT>(1);
        let (incoming_sender, incoming_receiver) = channel::<ReceiveT>(1);
        let result = ws.on_upgrade(move |socket| async move {
            let (client_sender, mut client_receiver) = new_client(socket);

            {
                let client_sender = client_sender.clone();
                tokio::spawn(async move {
                    while let Some(msg) = outgoing_receive.recv().await {
                        if let Err(e) = client_sender.send(msg).await {
                            error!("error sending to outgoing websocket channel: {e:?}");
                        }
                    }
                });
            }

            tokio::spawn(async move {
                while let Some(msg) = client_receiver.recv().await {
                    if let Err(e) = incoming_sender.send(msg).await {
                        error!("error sending to incoming websocket channel: {e:?}");
                    }
                }
            });

            let mut clients = clients.lock().unwrap();
            clients.push(client_sender);
        });
        (result, outgoing_send, incoming_receiver)
    }

    pub async fn broadcast(&self, msg: &SendT)
    where
        SendT: Clone,
    {
        let clients = self.clients.lock().unwrap().clone();
        // TODO parallelize
        for client in clients.iter() {
            if let Err(e) = client.send(msg.clone()).await {
                error!("error sending to outgoing websocket channel: {e:?}");
            }
        }
    }
}

fn new_client<SendT, ReceiveT>(socket: WebSocketStream) -> (Sender<SendT>, Receiver<ReceiveT>)
where
    SendT: Serialize + Send + 'static,
    ReceiveT: DeserializeOwned + Send + 'static,
{
    let (mut sink, mut stream) = socket.split();

    let (outgoing_sender, mut outgoing_receiver) = channel::<SendT>(1);
    let (incoming_sender, incoming_receiver) = channel::<ReceiveT>(1);

    tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            if let Some(msg) = match msg {
                poem::web::websocket::Message::Text(msg) => Some(Message::Text(msg)),
                poem::web::websocket::Message::Binary(msg) => Some(Message::Binary(msg)),
                _ => None,
            }
            .map(|msg| {
                msg.deserialize().unwrap_or_else(|e| {
                    error!("error deserializing incoming websocket message: {e:?}");
                    None
                })
            })
            .flatten()
            {
                if let Err(e) = incoming_sender.send(msg).await {
                    error!(
                        "error sending deserialized websocket message to incoming channel: {e:?}"
                    );
                }
            }
        }
        debug!("websocket disconnected");
    });

    tokio::spawn(async move {
        while let Some(msg) = outgoing_receiver.recv().await {
            match Message::serialize(&msg).map(|msg| match msg {
                Message::Text(msg) => poem::web::websocket::Message::Text(msg),
                Message::Binary(msg) => poem::web::websocket::Message::Binary(msg),
            }) {
                Ok(msg) => {
                    if let Err(e) = sink.send(msg).await {
                        error!("error sending to websocket: {e:?}");
                    }
                }
                Err(e) => error!("error serializing outgoing websocket message: {e:?}"),
            }
        }
    });

    (outgoing_sender, incoming_receiver)
}
