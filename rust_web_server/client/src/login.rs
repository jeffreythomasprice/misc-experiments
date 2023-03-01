use crate::rename::get_value_from_input_element;
use base64::Engine;
use log::*;
use shared::auth;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use yew::{function_component, html, use_state, Callback, Html, InputEvent, SubmitEvent};

#[function_component]
pub fn Login() -> Html {
    let username = use_state(|| "".to_string());
    let username_changed = {
        let state = username.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(x) = get_value_from_input_element(e) {
                state.set(x)
            }
        })
    };

    let password = use_state(|| "".to_string());
    let password_changed = {
        let state = password.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(x) = get_value_from_input_element(e) {
                state.set(x)
            }
        })
    };

    let submit = {
        let username = username.clone();
        let password = password.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            debug!(
                "TODO JEFF submit, username = {}, password = {}",
                *username, *password
            );

            let username = username.clone();
            let password = password.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match login(&username, &password).await {
                    Ok(_) => debug!(
                        "TODO login complete, should be sending a success or failure message"
                    ),
                    Err(e) => error!("TODO JEFF login error: {e:?}"),
                }
            });
        })
    };

    html! {
        <form
            onsubmit={submit}
        >
            <div class="row">
                <div class="col">
                    <label for="username">{ "Username" }</label>
                    <input name="username" type="text" oninput={username_changed} />
                </div>
            </div>
            <div class="row">
                <div class="col">
                    <label for="password">{ "Password" }</label>
                    <input name="password" type="password" oninput={password_changed} />
                </div>
            </div>
            <div class="row">
                <div class="col">
                    <button type="submit" class="button primary">{ "Login" }</button>
                </div>
            </div>
        </form>
    }
}

// TODO move to a service
async fn login(username: &str, password: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("failed to get window")?;

    // TODO don't hard-code url
    let request = Request::new_with_str_and_init(
        "http://localhost:8001/login",
        RequestInit::new()
            .method("POST")
            .mode(web_sys::RequestMode::Cors),
    )?;
    request
        .headers()
        .set("Authorization", &get_basic_auth_header(username, password))?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .dyn_into::<Response>()?;
    let response_body: auth::ResponseBody =
        serde_wasm_bindgen::from_value(JsFuture::from(response.json()?).await?)?;
    trace!("got jwt: {}", response_body.jwt);

    let storage = window
        .local_storage()?
        .ok_or("failed to get local storage")?;
    storage.set("jwt", response_body.jwt.as_str())?;
    trace!("saved jwt in local storage");

    Ok(())
}

// TODO move
fn get_basic_auth_header(username: &str, password: &str) -> String {
    // TODO url-encode username and password
    let response = format!(
        "Basic {}",
        base64::engine::general_purpose::URL_SAFE.encode(format!("{username}:{password}"))
    );
    trace!("TODO JEFF header = {}", response);
    response
}
