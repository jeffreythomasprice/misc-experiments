use log::*;

use leptos::*;
use shared::JsonResponse;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

mod fetch;
use fetch::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    let click = move |_| {
        set_count.update(|count| {
            *count += 1;
            ()
        });
    };

    view! {
        cx,
        <div>
            <p>"Clicks: " {move || count()}</p>
            <button on:click=click>"Click me!"</button>
        </div>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();

    spawn_local(async {
        if let Err(e) = example().await {
            console::log_2(&"error making request".into(), &e);
        }
    });

    mount_to_body(|cx| {
        view! {cx, <App/>}
    })
}

async fn example() -> Result<(), JsValue> {
    let response = RequestBuilder::new()
        .get()
        .url("http://127.0.0.1:8001/")
        .header("Accept", "text/plain")
        .build()?
        .launch()
        .await?;
    info!(
        "TODO JEFF status: {} {}",
        response.status(),
        response.status_text()
    );
    let response_body = response.text().await?;
    info!("TODO JEFF response body: {response_body}");

    let response = RequestBuilder::new()
        .get()
        .url("http://127.0.0.1:8001/json")
        .header("Accept", "application/json")
        .build()?
        .launch()
        .await?;
    info!(
        "TODO JEFF status: {} {}",
        response.status(),
        response.status_text()
    );
    let response_body: JsonResponse = response.json().await?;
    info!("TODO JEFF response body: {response_body:?}");

    Ok(())
}
