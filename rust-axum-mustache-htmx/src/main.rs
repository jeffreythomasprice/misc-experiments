use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    clicks: Arc<Mutex<u64>>,
    templates: Arc<Templates>,
}

struct Templates {
    templates: Mutex<RefCell<HashMap<String, Arc<Template>>>>,
}

impl Templates {
    pub fn new() -> mustache::Result<Templates> {
        Ok(Self {
            templates: Mutex::new(RefCell::new(HashMap::new())),
        })
    }

    pub fn render_click_form(&self, clicks: u64) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {
            clicks: u64,
        }

        self.render_page(self.render_template_string(
            "click form",
            include_str!("../templates/click-form.html"),
            &Data { clicks },
        )?)
    }

    pub fn render_click_response(&self, clicks: u64) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {
            clicks: u64,
        }

        self.render_template_string(
            "click response",
            include_str!("../templates/click-response.html"),
            &Data { clicks },
        )
    }

    fn render_page(&self, contents: String) -> mustache::Result<String> {
        #[derive(Serialize)]
        struct Data {
            contents: String,
        }

        let template = self.get_template("page", include_str!("../templates/page.html"))?;
        template.render_to_string(&Data { contents })
    }

    fn render_template_string<T>(
        &self,
        name: &str,
        source: &str,
        data: &T,
    ) -> mustache::Result<String>
    where
        T: Serialize,
    {
        let template = self.get_template(name, source)?;
        template.render_to_string(data)
    }

    fn get_template(&self, name: &str, source: &str) -> mustache::Result<Arc<Template>> {
        let mut templates = self.templates.lock().unwrap();
        let templates = templates.get_mut();
        match templates.get(name) {
            Some(result) => Ok(result.clone()),
            None => {
                let result = Arc::new(mustache::compile_str(source)?);
                templates.insert(name.to_string(), result.clone());
                Ok(result)
            }
        }
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
