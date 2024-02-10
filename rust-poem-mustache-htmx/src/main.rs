use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mustache::Template;
use poem::{
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{AddData, Tracing},
    post,
    web::{headers::ContentType, Data},
    EndpointExt, IntoResponse, Response, Route, Server,
};
use serde::Serialize;
use tracing::*;

#[derive(Debug)]
enum TemplateError {
    Compile,
    Render,
}

impl From<TemplateError> for StatusCode {
    fn from(_value: TemplateError) -> Self {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Clone)]
struct TemplateService {
    templates: Arc<Mutex<HashMap<String, Arc<Template>>>>,
}

impl TemplateService {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_with_source_str(
        &self,
        name: &str,
        source: &str,
    ) -> Result<Arc<Template>, TemplateError> {
        let mut templates = self.templates.lock().unwrap();
        Ok(match templates.entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(e) => e.get().clone(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let result = Arc::new(mustache::compile_str(source).map_err(|e| {
                    error!("failed to compile template {name}: {e:?}");
                    TemplateError::Compile
                })?);
                e.insert(result.clone());
                result
            }
        })
    }

    pub fn render_to_string_with_source_str<T>(
        &self,
        name: &str,
        source: &str,
        data: &T,
    ) -> Result<String, TemplateError>
    where
        T: Serialize,
    {
        let template = self.get_with_source_str(name, source)?;
        template.render_to_string(data).map_err(|e| {
            error!("failed to render template {name}: {e:?}");
            TemplateError::Render
        })
    }
}

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
) -> Result<Response, StatusCode> {
    #[derive(Serialize)]
    struct Data {
        clicks: u64,
    }
    Ok(page(
        &templates,
        &templates.render_to_string_with_source_str(
            "clicks",
            include_str!("./clicks.html"),
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
) -> Result<Response, StatusCode> {
    #[derive(Serialize)]
    struct Data {
        clicks: u64,
    }
    Ok(templates
        .render_to_string_with_source_str(
            "clicks-response",
            include_str!("./clicks-response.html"),
            &Data {
                clicks: clicks.click(),
            },
        )?
        .with_content_type(ContentType::html().to_string())
        .into_response())
}

#[handler]
fn index_css() -> Response {
    include_str!("../static/index.css")
        .with_content_type("text/css")
        .into_response()
}

#[handler]
fn htmx_js() -> Response {
    include_str!("../static/htmx/1.9.10/htmx.min.js")
        .with_content_type("text/javascript")
        .into_response()
}

#[handler]
fn htmx_ws_js() -> Response {
    include_str!("../static/htmx/1.9.10/ws.js")
        .with_content_type("text/javascript")
        .into_response()
}

fn page(templates: &TemplateService, content: &str) -> Result<String, TemplateError> {
    #[derive(Serialize)]
    struct Data<'a> {
        content: &'a str,
    }
    templates.render_to_string_with_source_str(
        "page",
        include_str!("./page.html"),
        &Data { content },
    )
}
