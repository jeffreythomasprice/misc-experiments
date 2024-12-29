use axum::extract::ws::{self, WebSocket};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};
use tokio::{spawn, sync::Mutex};
use tracing::*;
use uuid::Uuid;

use crate::{concurrent_hashmap::ConcurrentHashMap, HttpError};

#[derive(Debug, Deserialize)]
pub struct IncomingWebsocketMessage {
    pub message: String,
    #[serde(rename = "HEADERS")]
    pub headers: HashMap<String, Option<String>>,
}

#[derive(Clone)]
pub struct ActiveWebsocketConnection {
    pub id: Uuid,
    pub addr: SocketAddr,
    sink: Arc<Mutex<SplitSink<WebSocket, ws::Message>>>,
}

impl ActiveWebsocketConnection {
    pub fn new<IncomingMessageCallback, CloseCallback>(
        socket: WebSocket,
        addr: SocketAddr,
        incoming: IncomingMessageCallback,
        close: CloseCallback,
    ) -> Self
    where
        IncomingMessageCallback:
            Fn(&Self, IncomingWebsocketMessage) -> anyhow::Result<()> + Send + 'static,
        CloseCallback: Fn(&Self) -> anyhow::Result<()> + Send + 'static,
    {
        let (sink, mut stream) = socket.split();

        let result = Self {
            id: Uuid::new_v4(),
            addr,
            sink: Arc::new(Mutex::new(sink)),
        };

        {
            let result = result.clone();
            spawn(async move {
                while let Some(message) = stream.next().await {
                    match message {
                        Ok(ws::Message::Text(message)) => {
                            trace!(
                                "received websocket message, websocket: {:?}, message: {}",
                                result,
                                message
                            );
                            match serde_json::from_str(&message) {
                                Ok(message) => {
                                    if let Err(e) = incoming(&result, message) {
                                        error!("error in websocket message handler, websocket: {:?}, error: {:?}", result, e);
                                    }
                                }
                                Err(e) => error!("error deserializing incoming websocket message, websocket: {:?}, error: {:?}", result, e),
                            };
                        }
                        Ok(ws::Message::Binary(message)) => {
                            trace!("TODO received websocket binary message: {:?}", message);
                            todo!()
                        }
                        Ok(ws::Message::Close(_)) => {
                            debug!("websocket closed: {:?}", result);
                            if let Err(e) = close(&result) {
                                error!("error in websocket close handler while handling close event, websocket: {:?}, error: {:?}", result,e);
                            }
                            return;
                        }
                        Ok(ws::Message::Ping(_)) | Ok(ws::Message::Pong(_)) => (),
                        Err(e) => error!("error receiving websocket message: {:?}", e),
                    }
                }
                trace!("websocket loop closed, websocket: {:?}", result);
            });
        }

        result
    }

    async fn send(&self, s: String) -> anyhow::Result<()> {
        let sink = &mut *self.sink.lock().await;
        trace!("sending to {:?}, message={}", self, s);
        sink.send(ws::Message::Text(s)).await?;
        Ok(())
    }
}

impl Debug for ActiveWebsocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveWebsocketConnection")
            .field("id", &self.id)
            .field("addr", &self.addr)
            .finish()
    }
}

#[derive(Clone)]
pub struct WebSockets {
    websockets: ConcurrentHashMap<Uuid, ActiveWebsocketConnection>,
}

impl WebSockets {
    pub fn new() -> Self {
        Self {
            websockets: ConcurrentHashMap::new(),
        }
    }

    pub async fn insert(&mut self, ws: ActiveWebsocketConnection) {
        info!("registered new active websocket connection: {:?}", ws);
        self.websockets.insert(ws.id, ws).await;
    }

    pub async fn remove(&mut self, ws: ActiveWebsocketConnection) {
        info!("removing websocket connection: {:?}", ws);
        self.websockets.remove(&ws.id).await;
    }

    pub async fn broadcast(&mut self, message: String) -> Result<(), HttpError> {
        info!("broadcasting {}", message);
        for ws in self.websockets.values().await {
            if let Err(e) = ws.send(message.clone()).await {
                error!(
                    "error sending message to websocket: {:?}, error: {:?}",
                    ws, e
                );
            }
        }
        Ok(())
    }
}
