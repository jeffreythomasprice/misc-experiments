mod templates;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use include_dir::include_dir;
use mustache::Template;
use poem::{
    endpoint, get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    post,
    web::{headers::ContentType, Data},
    Endpoint, EndpointExt, IntoResponse, Response, Route, Server,
};
use serde::Serialize;
use templates::TemplateError;
use tracing::*;

use crate::templates::TemplateService;

// TODO use me
static STATIC_DIR: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

type HttpError = StatusCode;

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
        .at("/index.css", get(index_css))
        .at("/htmx.min.js", get(htmx_js))
        .at("/ws.js", get(htmx_ws_js))
        .at("/foo", endpoint::make(|_| async { "test" }))
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
    struct Data {
        clicks: u64,
    }
    Ok(page(
        &templates,
        &templates.render(
            "clicks.html",
            &Data {
                clicks: clicks.get(),
            },
        )?,
    )?
    .with_content_type(ContentType::html().to_string())
    .into_response())
}

#[handler]
fn click(
    templates: Data<&TemplateService>,
    clicks: Data<&ClicksService>,
) -> Result<Response, HttpError> {
    #[derive(Serialize)]
    struct Data {
        clicks: u64,
    }
    Ok(templates
        .render(
            "clicks-response.html",
            &Data {
                clicks: clicks.click(),
            },
        )?
        .with_content_type(ContentType::html().to_string())
        .into_response())
}

#[handler]
fn index_css() -> Response {
    // TODO use STATIC_DIR
    include_str!("../static/index.css")
        .with_content_type("text/css")
        .into_response()
}

#[handler]
fn htmx_js() -> Response {
    // TODO use STATIC_DIR
    include_str!("../static/htmx/1.9.10/htmx.min.js")
        .with_content_type("text/javascript")
        .into_response()
}

#[handler]
fn htmx_ws_js() -> Response {
    // TODO use STATIC_DIR
    include_str!("../static/htmx/1.9.10/ws.js")
        .with_content_type("text/javascript")
        .into_response()
}

fn page(templates: &TemplateService, content: &str) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data<'a> {
        content: &'a str,
    }
    templates.render("page.html", &Data { content })
}

fn static_file(path: &str) -> impl Endpoint {
    endpoint::make(|_| -> Result<Response, _> {
        async {
            let file = STATIC_DIR.get_file(path).ok_or_else(|| {
                error!("no such static file: {path}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?;
            let contents = file.contents();
            let path = path.to_lowercase();
            let content_type = if path.ends_with(".html") || path.ends_with(".htm") {
                "text/html"
            } else if path.ends_with(".js") {
                "text/javascript"
            } else if path.ends_with(".css") {
                "text/css"
            } else {
                "text/plain"
            };
            Ok(contents.with_content_type(content_type).into_response())
        }
    })
}

impl From<TemplateError> for HttpError {
    fn from(_value: TemplateError) -> Self {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
