mod http_utils;
mod static_files;
mod templates;
mod websockets;

use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    post,
    web::{websocket::WebSocket, Data},
    EndpointExt, IntoResponse, Request, Response, Route, Server,
};
use serde::Serialize;
use static_files::static_file;
use std::sync::{Arc, Mutex};
use templates::TemplateError;
use websockets::WebsocketService;

use tracing::*;

use crate::{
    http_utils::{to_html_response, HttpError},
    templates::TemplateService,
    websockets::{websockets_form, WebsocketClient},
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
        .with(AddData::new(ClicksService::new()))
        .with(AddData::new(WebsocketService::new()));
    Server::new(TcpListener::bind("127.0.0.1:8000"))
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
async fn ws(
    req: &Request,
    ws: WebSocket,
    ws_service: Data<&WebsocketService>,
    templates: Data<&TemplateService>,
) -> impl IntoResponse {
    debug!("remote_addr={}", req.remote_addr());
    let (result, _sender, mut receiver) = ws_service.on_upgrade(ws, templates.clone());

    let ws_service = ws_service.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            ws_service.broadcast(msg).await;
        }
    });

    result
}

fn click_text(templates: &TemplateService, clicks: u64) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data {
        clicks: u64,
    }
    templates.render("clicks-response.html", &Data { clicks })
}
