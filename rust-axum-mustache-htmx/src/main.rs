mod db;
mod templates;

use std::sync::Arc;

use axum::{
    extract::{FromRef, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};

use db::*;
use serde::{Deserialize, Serialize};

use templates::*;
use tracing::*;

#[derive(Clone)]
struct AppState {
    templates: Arc<TemplateService>,
    db: Arc<DbService>,
}

impl FromRef<AppState> for Arc<TemplateService> {
    fn from_ref(input: &AppState) -> Self {
        input.templates.clone()
    }
}

impl FromRef<AppState> for Arc<DbService> {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

#[derive(Debug)]
enum ResponseError {
    Mustache(mustache::Error),
    Sqlx(sqlx::Error),
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

impl From<sqlx::Error> for ResponseError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
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

    // TODO error logging instead of unwrapping everything

    let db = DbService::new().await.unwrap();

    // TODO axum tracing config?

    let app = Router::new()
        .route("/", get(index))
        .route("/login", post(login))
        .route("/index.css", get(index_css))
        .with_state(AppState {
            templates: Arc::new(TemplateService::new().unwrap()),
            db: Arc::new(db),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index(
    State(templates): State<Arc<TemplateService>>,
) -> Result<Html<String>, ResponseError> {
    Ok(Html(
        templates
            .page(include_str!("../templates/login-form.html"))?
            .render_to_string(&())?,
    ))
}

async fn login(
    State(templates): State<Arc<TemplateService>>,
    State(db): State<Arc<DbService>>,
    Form(form): Form<LoginForm>,
) -> Result<Html<String>, ResponseError> {
    if db.check_password(&form.username, &form.password).await? {
        trace!("login success for {}", form.username);
        Ok(Html(
            templates
                .page(include_str!("../templates/logged-in.html"))?
                .render_to_string(&())?,
        ))
    } else {
        trace!("login failed for {}", form.username);
        Ok(Html(error_response_str(
            &templates,
            "TODO error message here",
        )?))
    }
}

async fn index_css() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/css".parse().unwrap());
    (headers, include_str!("../static/index.css"))
}

#[derive(Serialize)]
struct ErrorMessage {
    pub message: String,
}

fn error_response(
    templates: &TemplateService,
    messages: &Vec<ErrorMessage>,
) -> Result<String, ResponseError> {
    #[derive(Serialize)]
    struct Data<'a> {
        messages: &'a Vec<ErrorMessage>,
    }
    Ok(templates
        .snippet(include_str!("../templates/error-response.html"))?
        .render_to_string(&Data { messages })?)
}

fn error_response_str(templates: &TemplateService, message: &str) -> Result<String, ResponseError> {
    error_response(
        templates,
        &vec![ErrorMessage {
            message: message.to_string(),
        }],
    )
}
