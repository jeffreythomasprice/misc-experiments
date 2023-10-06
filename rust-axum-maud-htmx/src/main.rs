use std::error::Error;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use maud::{html, Markup, DOCTYPE};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        .route("/click", post(click));

    axum::Server::bind(&"127.0.0.1:8000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn index() -> Markup {
    html_page(|| {
        html! {
            h1 { "Hello, World!" }
            button hx-post="/click" hx-target="#clickResults" { "Click Me" }
            div id="clickResults";
        }
    })
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
