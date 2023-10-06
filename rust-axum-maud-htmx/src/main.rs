use std::error::Error;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;
use tokio::sync::Mutex;

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
        div { (message) }
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
        form hx-post="/login" {
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
                script src="https://unpkg.com/htmx.org@1.9.6" {}
            }
            body {
                (f())
            }
        }
    }
}
