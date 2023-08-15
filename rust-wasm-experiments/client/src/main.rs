use console_log;
use log::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{console, Request, RequestInit, Response};
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! {
        <div>
            <p>{ "Hello, World!" }</p>
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

    yew::Renderer::<App>::new().render();
}

async fn example() -> Result<(), JsValue> {
    let mut request_init = RequestInit::new();
    request_init.method("GET");
    request_init.mode(web_sys::RequestMode::Cors);
    let request = Request::new_with_str_and_init("http://127.0.0.1:8001/", &request_init)?;
    request.headers().set("Accept", "text/plain")?;

    let window = web_sys::window().unwrap();
    let response: Response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .dyn_into()?;
    console::log_2(&"TODO JEFF response".into(), &response);

    let response_body = JsFuture::from(response.text()?).await?;
    console::log_2(&"TODO JEFF response body".into(), &response_body);

    Ok(())
}
