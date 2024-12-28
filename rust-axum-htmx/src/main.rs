use std::{
    collections::HashMap,
    fmt::Debug,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use envconfig::Envconfig;
use mustache::Template;
use serde::Serialize;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "HOST")]
    pub host: String,

    #[envconfig(from = "PORT")]
    pub port: u16,
}

struct HttpError {
    status: StatusCode,
    message: String,
}

impl From<anyhow::Error> for HttpError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("{value:}"),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}

#[derive(Clone)]
struct AppState {
    templates: Arc<Mutex<HashMap<String, Arc<Template>>>>,
    clicks: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
struct Index {
    content: String,
}

#[derive(Serialize)]
struct Counter {
    clicks: u64,
}

impl AppState {
    fn new() -> Self {
        Self {
            templates: Arc::new(Mutex::new(HashMap::new())),
            clicks: Arc::new(Mutex::new(0)),
        }
    }

    fn template_path<P>(&mut self, path: P) -> Result<Arc<Template>, HttpError>
    where
        P: AsRef<Path> + Debug,
    {
        // TODO replace with a FallableCache system?
        let templates = &mut *self.templates.lock().unwrap();
        let key = format!("{:?}", path);
        match templates.entry(key) {
            std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                Ok(occupied_entry.get().clone())
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                let result = Arc::new(mustache::compile_path(&path).map_err(|e| {
                    anyhow!(
                        "error compiling template from path: {:?}, error: {:?}",
                        path,
                        e
                    )
                })?);
                vacant_entry.insert(result.clone());
                Ok(result)
            }
        }
    }

    fn template_path_to_string<P, T>(&mut self, path: P, data: &T) -> Result<String, HttpError>
    where
        P: AsRef<Path> + Debug,
        T: Serialize,
    {
        let template = self.template_path(path)?;
        let result = template
            .render_to_string(data)
            .map_err(|e| anyhow!("error rendering template: {e:?}"))?;
        Ok(result)
    }

    fn get_clicks(&self) -> u64 {
        *self.clicks.lock().unwrap()
    }

    fn click(&mut self) -> u64 {
        let clicks = &mut *self.clicks.lock().unwrap();
        *clicks += 1;
        *clicks
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::from_filename("local.env")?;
    let config = Config::init_from_env().unwrap();

    let state = AppState::new();

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(index))
        .route("/click", post(click))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let serve_result = axum::serve(listener, app);
    info!("listening at {}", addr);
    serve_result.await?;

    Ok(())
}

async fn index(State(mut state): State<AppState>) -> Result<impl IntoResponse, HttpError> {
    let content = state.template_path_to_string(
        "templates/counter.html",
        &Counter {
            clicks: state.get_clicks(),
        },
    )?;
    Ok(Html(state.template_path_to_string(
        "templates/index.html",
        &Index { content },
    )?))
}

async fn click(State(mut state): State<AppState>) -> Result<impl IntoResponse, HttpError> {
    let clicks = state.click();
    info!("click, new counter: {}", clicks);
    Ok(Html(state.template_path_to_string(
        "templates/counter.html",
        &Counter { clicks },
    )?))
}
