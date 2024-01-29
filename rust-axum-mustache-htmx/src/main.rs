mod templates;

use std::sync::{Arc, Mutex};

use axum::{
    extract::{FromRef, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{AppendHeaders, Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};

use serde::Deserialize;
use templates::*;
use tracing::*;

/*
TODO next:
- "login" form
    - some new htmx
    - cookies?
    - sql for user db
    - FromRef for the db part (user service?) from the main app state
*/

#[derive(Clone)]
struct AppState {
    templates: Arc<Templates>,
}

impl FromRef<AppState> for Arc<Templates> {
    fn from_ref(input: &AppState) -> Self {
        input.templates.clone()
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

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
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
        .route("/login", post(login))
        .route("/index.css", get(index_css))
        .with_state(AppState {
            templates: Arc::new(Templates::new().unwrap()),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index(State(templates): State<Arc<Templates>>) -> Result<Html<String>, ResponseError> {
    Ok(Html(templates.login_form()?))
}

async fn login(
    State(templates): State<Arc<Templates>>,
    Form(form): Form<LoginForm>,
) -> Result<Html<String>, ResponseError> {
    debug!(
        "TODO username: {}, password: {}",
        form.username, form.password
    );
    todo!()
}

async fn index_css() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/css".parse().unwrap());
    (headers, include_str!("../static/index.css"))
}
