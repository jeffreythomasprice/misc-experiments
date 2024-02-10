use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use poem::{
    web::websocket::{Message, WebSocket, WebSocketStream},
    IntoResponse,
};
use serde::{Deserialize, Serialize};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::*;

use crate::templates::{TemplateError, TemplateService};

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
        templates: TemplateService,
    ) -> (impl IntoResponse, Sender<String>, Receiver<String>) {
        let clients = self.clients.clone();
        let templates = templates.clone();
        let (outgoing_send, mut outgoing_receive) = channel(1);
        let (incoming_sender, incoming_receiver) = channel::<String>(1);
        let result = ws.on_upgrade(move |socket| async move {
            let client = WebsocketClient::new(socket, templates, incoming_sender);

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
    templates: TemplateService,
    outgoing_sender: Sender<String>,
}

impl WebsocketClient {
    pub fn new(
        socket: WebSocketStream,
        templates: TemplateService,
        incoming_sender: Sender<String>,
    ) -> Self {
        let (mut sink, mut stream) = socket.split();

        let (outgoing_sender, mut outgoing_receiver) = channel::<String>(1);

        {
            let templates = templates.clone();
            let outgoing_sender = outgoing_sender.clone();
            tokio::spawn(async move {
                while let Some(Ok(msg)) = stream.next().await {
                    match std::str::from_utf8(msg.as_bytes()) {
                        Ok(msg) => {
                            trace!("received websocket message: {msg}");

                            #[derive(Deserialize)]
                            struct HXWebsocketMessage {
                                #[serde(rename = "ws-message")]
                                pub message: String,
                            }
                            match serde_json::from_str::<HXWebsocketMessage>(msg) {
                                Ok(HXWebsocketMessage { message: msg }) => {
                                    debug!("received websocket message: {msg}");
                                    if let Err(e) = incoming_sender.send(msg).await {
                                        error!("error sending incoming websocket message to channel: {e:?}");
                                    }
                                }
                                Err(e) => {
                                    error!("error deserializing websocket message: {e:?}");
                                }
                            };

                            send_reset_form_input(&outgoing_sender, &templates).await;
                        }
                        Err(e) => {
                            error!("received websocket message, but didn't look like utf8: {e:?}")
                        }
                    };
                }
                debug!("websocket disconnected");
            });
        }

        tokio::spawn(async move {
            while let Some(msg) = outgoing_receiver.recv().await {
                if let Err(e) = sink.send(Message::Text(msg)).await {
                    error!("error sending to websocket: {e:?}");
                }
            }
        });

        Self {
            templates,
            outgoing_sender,
        }
    }

    pub async fn send(&self, msg: String) {
        send_text_message(&self.outgoing_sender, &self.templates, msg).await;
    }
}

async fn send_reset_form_input(outgoing_sender: &Sender<String>, templates: &TemplateService) {
    if let Ok(msg) = websockets_input(templates) {
        put_on_outgoing(outgoing_sender, msg).await;
    }
}

async fn send_text_message(
    outgoing_sender: &Sender<String>,
    templates: &TemplateService,
    msg: String,
) {
    if let Ok(msg) = websockets_message(templates, &msg) {
        put_on_outgoing(outgoing_sender, msg).await;
    }
}

async fn put_on_outgoing(outgoing_sender: &Sender<String>, msg: String) {
    if let Err(e) = outgoing_sender.send(msg).await {
        error!("error sending to outgoing websocket channel: {e:?}");
    }
}

pub fn websockets_form(templates: &TemplateService) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data<'a> {
        input: &'a str,
    }
    let input = websockets_input(templates)?;
    templates.render("ws-form.html", &Data { input: &input })
}

fn websockets_input(templates: &TemplateService) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data {}
    templates.render("ws-input.html", &Data {})
}

fn websockets_message(templates: &TemplateService, msg: &str) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data<'a> {
        content: &'a str,
    }
    templates.render("ws-message.html", &Data { content: msg })
}
