use axum::{
    body::Body,
    extract::{FromRef, Request, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::*;
use uuid::Uuid;
mod errors;
use errors::*;
mod templates;
use templates::*;

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
    Ok(t.counter_page(*count).await?)
}

async fn click(
    State(app): State<AppState>,
    State(t): State<Templates>,
) -> HttpResult<impl IntoResponse> {
    let mut count = app.count.lock().await;
    *count += 1;
    Ok(t.click_response(*count).await?)
}
