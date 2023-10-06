use std::error::Error;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{AppendHeaders, IntoResponse};
use axum::routing::{get, post};
use axum::{Form, Router};
use include_dir::{include_dir, Dir};
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;
use tokio::sync::Mutex;

static ASSETS_DIR: Dir = include_dir!("assets");

#[derive(Clone)]
struct UsersService {}

impl UsersService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<bool, Box<dyn Error + Send>> {
        println!("checking login credentials: {username}");
        let result = password == "password";
        println!("result = {result}");
        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let users_service = UsersService::new();

    let clicks = Arc::new(Mutex::<u32>::new(0));

    let click = {
        // let clicks = clicks.clone();
        || async move {
            let mut clicks = clicks.lock().await;
            *clicks += 1;
            html! {
                div { (format!("Clicks: {}", *clicks)) }
            }
        }
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/click", post(click))
        .route("/login", post(login))
        .route("/*path", get(asset_file))
        .with_state(users_service);

    axum::Server::bind(&"127.0.0.1:8000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct LoginFormData {
    username: String,
    password: String,
}

#[axum::debug_handler]
async fn login(
    State(users_service): State<UsersService>,
    Form(request): Form<LoginFormData>,
) -> Markup {
    match users_service
        .check_credentials(&request.username, &request.password)
        .await
    {
        Ok(true) => {
            html! {
                div { "TODO login response" }
            }
        }
        Ok(false) => error_response("Bad credentials").await,
        Err(e) => {
            println!("error checking user credentials: {e:?}");
            error_response("Error").await
        }
    }
}

async fn error_response(message: &str) -> Markup {
    html! {
        div hx-swap-oob="innerHTML:#errorMessages" {
            div class="error" { (message) }
        }
    }
}

async fn index() -> Markup {
    html_page(|| {
        html! {
            h1 { "Hello, World!" }
            button hx-post="/click" hx-target="#clickResults" { "Click Me" }
            div id="clickResults" {}
            (login_form())
        }
    })
}

fn login_form() -> Markup {
    html! {
        form hx-post="/login" hx-swap="none" {
            div {
                label for="username" { "Username:" }
                input name="username" placeholder="Username" type="text" {}
            }
            div {
                label for="password" { "Password:" }
                input name="password" placeholder="Password" type="password" {}
            }
            button type="submit" { "Log In" }
        }
        div id="errorMessages" {}
    }
}

fn html_page<F>(f: F) -> Markup
where
    F: Fn() -> Markup,
{
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                link rel="stylesheet" href="index.css";
                script src="https://unpkg.com/htmx.org@1.9.6" {}
                script {
                    "htmx.logAll();"
                }
            }
            body {
                (f())
            }
        }
    }
}

async fn asset_file(Path(path): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let path = path.trim_start_matches("/");
    let mime_type = mime_guess::from_path(path).first_or_text_plain();
    let headers = AppendHeaders([(
        CONTENT_TYPE,
        HeaderValue::from_str(mime_type.as_ref()).or_else(|e| {
            println!("error getting mime type for {path}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        })?,
    )]);
    match ASSETS_DIR.get_file(path) {
        Some(file) => Ok((headers, file.contents())),
        None => {
            println!("no such file found {path}");
            Err(StatusCode::NOT_FOUND)?
        }
    }
}
