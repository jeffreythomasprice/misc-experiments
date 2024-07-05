use leptos::*;
use log::*;
use shared::Example;

#[component]
fn App() -> impl IntoView {
    create_resource(||(), |_| async {
        match get_example().await {
            Ok(response) => info!("example response: {response:?}"),
            Err(e) => error!("error loading example data: {e:?}"),
        }
    });
    view! { <div>Hello, World!</div> }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    mount_to_body(|| App())
}

async fn get_example() -> anyhow::Result<Example> {
    Ok(reqwest::get("http://localhost:8001/json").await?.json::<Example>().await?)
}