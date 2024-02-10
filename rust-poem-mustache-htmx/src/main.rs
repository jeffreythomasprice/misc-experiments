mod http_utils;
mod static_files;
mod templates;

use std::sync::{Arc, Mutex};

use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    post,
    web::{headers::ContentType, Data},
    EndpointExt, IntoResponse, Response, Route, Server,
};
use serde::Serialize;
use static_files::static_file;
use templates::TemplateError;

use crate::{http_utils::HttpError, templates::TemplateService};

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
    let content = click_text(&templates, clicks.get())?;
    let content = templates.render("clicks.html", &Data { content: &content })?;
    Ok(page(&templates, &content)?
        .with_content_type(ContentType::html().to_string())
        .into_response())
}

#[handler]
fn click(
    templates: Data<&TemplateService>,
    clicks: Data<&ClicksService>,
) -> Result<Response, HttpError> {
    Ok(click_text(&templates, clicks.click())?
        .with_content_type(ContentType::html().to_string())
        .into_response())
}

fn click_text(templates: &TemplateService, clicks: u64) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data {
        clicks: u64,
    }
    templates.render("clicks-response.html", &Data { clicks })
}

fn page(templates: &TemplateService, content: &str) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data<'a> {
        content: &'a str,
    }
    templates.render("page.html", &Data { content })
}
