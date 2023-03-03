use crate::dom_utils::{get_local_storage, get_value_from_input_element_event, get_window};
use base64::Engine;
use log::*;
use shared::auth;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use yew::{
    function_component, html, platform::spawn_local, use_state, Callback, Html, InputEvent,
    Properties, SubmitEvent,
};

const LOCAL_STORAGE_KEY: &str = "jwt";

#[derive(PartialEq, Properties)]
pub struct LoginProps {
    pub login_success: Callback<()>,
}

#[function_component]
pub fn Login(props: &LoginProps) -> Html {
    let username = use_state(|| "".to_string());
    let username_changed = {
        let state = username.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(x) = get_value_from_input_element_event(e) {
                state.set(x)
            }
        })
    };

    let password = use_state(|| "".to_string());
    let password_changed = {
        let state = password.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(x) = get_value_from_input_element_event(e) {
                state.set(x)
            }
        })
    };

    let submit = {
        let login_success = props.login_success.clone();
        let username = username.clone();
        let password = password.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            trace!("logging in {}", *username);

            let login_success = login_success.clone();
            let username = username.clone();
            let password = password.clone();
            spawn_local(async move {
                match login(&username, &password).await {
                    Ok(_) => {
                        trace!("login success");
                        login_success.emit(());
                    }
                    Err(e) => todo!("show an error message for login failure"),
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

pub fn get_authorization_header() -> Result<Option<String>, JsValue> {
    Ok(get_local_storage()?.get(LOCAL_STORAGE_KEY)?)
}

// TODO move to a service
async fn login(username: &str, password: &str) -> Result<(), JsValue> {
    let request = Request::new_with_str_and_init("/api/login", RequestInit::new().method("POST"))?;
    request
        .headers()
        .set("Authorization", &get_basic_auth_header(username, password))?;
    let response = JsFuture::from(get_window()?.fetch_with_request(&request))
        .await?
        .dyn_into::<Response>()?;
    let response_body: auth::ResponseBody =
        serde_wasm_bindgen::from_value(JsFuture::from(response.json()?).await?)?;
    trace!("got jwt: {}", response_body.jwt);

    get_local_storage()?.set(LOCAL_STORAGE_KEY, response_body.jwt.as_str())?;
    trace!("saved jwt in local storage");

    Ok(())
}

// TODO move
fn get_basic_auth_header(username: &str, password: &str) -> String {
    // TODO url-encode username and password
    format!(
        "Basic {}",
        base64::engine::general_purpose::URL_SAFE.encode(format!("{username}:{password}"))
    )
}
