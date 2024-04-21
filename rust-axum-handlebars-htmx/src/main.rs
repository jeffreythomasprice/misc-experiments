use std::sync::Arc;

use axum::{
    body::Body,
    extract::{FromRef, Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use handlebars::Handlebars;
use serde::Serialize;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::*;
use uuid::Uuid;

struct HttpError {
    status_code: StatusCode,
    message: String,
}

impl From<anyhow::Error> for HttpError {
    fn from(value: anyhow::Error) -> Self {
        HttpError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: value.to_string(),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, self.message).into_response()
    }
}

type HttpResult<T> = Result<T, HttpError>;

#[derive(Clone)]
struct Templates {
    h: Arc<Handlebars<'static>>,
}

#[derive(Clone)]
struct AppState {
    templates: Templates,
    count: Arc<Mutex<u64>>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            templates: Templates::new()?,
            count: Arc::new(Mutex::new(0)),
        })
    }
}

impl FromRef<AppState> for Templates {
    fn from_ref(input: &AppState) -> Self {
        input.templates.clone()
    }
}

impl Templates {
    pub fn new() -> anyhow::Result<Self> {
        let mut result = Handlebars::new();
        result.register_template_string("page", include_str!("../templates/page.html"))?;
        result.register_template_string("counter", include_str!("../templates/counter.html"))?;
        result.register_template_string(
            "click-response",
            include_str!("../templates/click-response.html"),
        )?;
        Ok(Self {
            h: Arc::new(result),
        })
    }

    pub fn counter_page(&self, count: u64) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data {
            count: u64,
        }
        let content = self.h.render("counter", &Data { count })?;
        self.page(&content)
    }

    pub fn click_response(&self, count: u64) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data {
            count: u64,
        }
        let content = self.h.render("click-response", &Data { count })?;
        self.fragment(content)
    }

    fn page<'a>(&self, content: &'a str) -> anyhow::Result<(HeaderMap, String)> {
        #[derive(Serialize)]
        struct Data<'a> {
            content: &'a str,
        }
        self.fragment(self.h.render("page", &Data { content })?)
    }

    fn fragment(&self, content: String) -> anyhow::Result<(HeaderMap, String)> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/html; charset=utf-8".parse()?);
        Ok((headers, content))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("experiment=trace,debug")
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/click", post(click));

    let app = app.with_state(AppState::new()?);

    let app = app.layer(
        TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            let request_id = Uuid::new_v4();
            span!(
                Level::DEBUG,
                "request",
                method = tracing::field::display(request.method()),
                uri = tracing::field::display(request.uri()),
                request_id = tracing::field::display(request_id),
            )
        }),
    );

    let addr = "0.0.0.0:8000";
    info!("listinging on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}

async fn index(
    State(app): State<AppState>,
    State(t): State<Templates>,
) -> HttpResult<impl IntoResponse> {
    let count = app.count.lock().await;
    Ok(t.counter_page(*count)?)
}

async fn click(
    State(app): State<AppState>,
    State(t): State<Templates>,
) -> HttpResult<impl IntoResponse> {
    let mut count = app.count.lock().await;
    *count += 1;
    Ok(t.click_response(*count)?)
}
