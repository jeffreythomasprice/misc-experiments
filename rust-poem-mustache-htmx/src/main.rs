mod http_utils;
mod static_files;
mod templates;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

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
use serde::Serialize;
use static_files::static_file;
use templates::TemplateError;
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
fn ws(ws: WebSocket) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        debug!("websocket connected");

        let (mut sink, mut stream) = socket.split();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                match std::str::from_utf8(msg.as_bytes()) {
                    Ok(msg) => debug!("received websocket message: {}", msg),
                    Err(e) => {
                        error!("received websocket message, but didn't look like utf8: {e:?}")
                    }
                };
            }
            debug!("websocket disconnected");
        });

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Err(e) = sink.send(Message::Text("Hello from server!".into())).await {
                error!("error sending to websocket: {e:?}");
            }
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
    struct Data {}
    templates.render("ws-form.html", &Data {})
}
