mod http_utils;
mod static_files;
mod templates;

use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    post,
    web::{
        websocket::{Message, WebSocket},
        Data,
    },
    EndpointExt, IntoResponse, Response, Route, Server,
};
use serde::{Deserialize, Serialize};
use static_files::static_file;
use templates::TemplateError;
use tokio::sync::mpsc::{channel, Sender};
use tracing::*;

use crate::{
    http_utils::{to_html_response, HttpError},
    templates::TemplateService,
};

#[derive(Clone)]
struct ClicksService {
    data: Arc<Mutex<u64>>,
}

impl ClicksService {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get(&self) -> u64 {
        let data = self.data.lock().unwrap();
        *data
    }

    pub fn click(&self) -> u64 {
        let mut data = self.data.lock().unwrap();
        *data += 1;
        *data
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter("experiment=trace,poem=debug,debug")
        .init();

    let app = Route::new()
        .at("/", get(index))
        .at("/click", post(click))
        .at("/ws", get(ws))
        .at("/index.css", get(static_file("index.css")))
        .at("/htmx.min.js", get(static_file("htmx/1.9.10/htmx.min.js")))
        .at("/ws.js", get(static_file("htmx/1.9.10/ws.js")))
        .with(Tracing)
        .with(AddData::new(TemplateService::new()))
        .with(AddData::new(ClicksService::new()));
    Server::new(TcpListener::bind("0.0.0.0:8000"))
        .run(app)
        .await
}

#[handler]
fn index(
    templates: Data<&TemplateService>,
    clicks: Data<&ClicksService>,
) -> Result<Response, HttpError> {
    #[derive(Serialize)]
    struct Data<'a> {
        content: &'a str,
    }
    let clicks_content = click_text(&templates, clicks.get())?;
    let clicks_content = templates.render(
        "clicks.html",
        &Data {
            content: &clicks_content,
        },
    )?;

    let ws_content = websockets_form(&templates)?;

    Ok(to_html_response(
        templates.render_page(&format!("{clicks_content}{ws_content}"))?,
    ))
}

#[handler]
fn click(
    templates: Data<&TemplateService>,
    clicks: Data<&ClicksService>,
) -> Result<Response, HttpError> {
    let content = click_text(&templates, clicks.click())?;
    Ok(to_html_response(content))
}

#[handler]
fn ws(ws: WebSocket, templates: Data<&TemplateService>) -> impl IntoResponse {
    let templates = templates.clone();
    ws.on_upgrade(|socket| async move {
        debug!("websocket connected");

        let (mut sink, mut stream) = socket.split();

        let (outgoing_sender, mut outgoing_receiver) = channel::<String>(1);

        async fn put_on_outgoing(sender: &Sender<String>, msg: String) {
            if let Err(e) = sender.send(msg).await {
                error!("error sending to outgoing websocket channel: {e:?}");
            }
        }

        async fn send_reset_form_input(templates: &TemplateService, sender: &Sender<String>) {
            if let Ok(msg) = websockets_input(templates) {
                put_on_outgoing(sender, msg).await;
            }
        }

        async fn send_text_message(
            templates: &TemplateService,
            sender: &Sender<String>,
            msg: String,
        ) {
            if let Ok(msg) = websockets_message(templates, &msg) {
                put_on_outgoing(sender, msg).await;
            }
        }

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

                                    // TODO testing
                                    send_text_message(
                                        &templates,
                                        &outgoing_sender,
                                        format!("replying to: {msg}"),
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    error!("error deserializing websocket message: {e:?}");
                                }
                            };

                            send_reset_form_input(&templates, &outgoing_sender).await;
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

        // TODO testing?
        tokio::spawn(async move {
            send_text_message(
                &templates,
                &outgoing_sender,
                "Hello from server!".to_string(),
            )
            .await;
        });
    })
}

fn click_text(templates: &TemplateService, clicks: u64) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data {
        clicks: u64,
    }
    templates.render("clicks-response.html", &Data { clicks })
}

fn websockets_form(templates: &TemplateService) -> Result<String, TemplateError> {
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
