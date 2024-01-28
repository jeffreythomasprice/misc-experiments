use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use mustache::Template;
use serde::Serialize;
use tracing::*;

#[derive(Clone)]
struct AppState {
    clicks: Arc<Mutex<u64>>,
    templates: Arc<Templates>,
}

struct Templates {
    page: Template,
    clicks_form: Template,
    clicks_response: Template,
}

impl Templates {
    pub fn new() -> mustache::Result<Templates> {
        Ok(Self {
            page: mustache::compile_str(include_str!("../templates/page.html"))?,
            clicks_form: mustache::compile_str(include_str!("../templates/click-form.html"))?,
            clicks_response: mustache::compile_str(include_str!(
                "../templates/click-response.html"
            ))?,
        })
    }

    pub fn render_click_form(&self, clicks: u64) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {
            clicks: u64,
        }
        self.render_page(self.clicks_form.render_to_string(&Data { clicks })?)
    }

    pub fn render_click_response(&self, clicks: u64) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {
            clicks: u64,
        }
        self.render_page(self.clicks_response.render_to_string(&Data { clicks })?)
    }

    fn render_page(&self, contents: String) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {
            contents: String,
        }
        self.page.render_to_string(&Data { contents })
    }
}

#[derive(Debug)]
enum ResponseError {
    Mustache(mustache::Error),
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        error!("response error: {self:?}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl From<mustache::Error> for ResponseError {
    fn from(value: mustache::Error) -> Self {
        Self::Mustache(value)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("experiment=trace".parse().unwrap()),
        )
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/click", post(click))
        .with_state(AppState {
            clicks: Arc::new(Mutex::new(0)),
            templates: Arc::new(Templates::new().unwrap()),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, ResponseError> {
    let clicks = state.clicks.lock().unwrap();
    Ok(Html(state.templates.render_click_form(*clicks)?))
}

async fn click(State(state): State<AppState>) -> Result<Html<String>, ResponseError> {
    let mut clicks = state.clicks.lock().unwrap();
    *clicks += 1;
    Ok(Html(state.templates.render_click_response(*clicks)?))
}
