use crate::rename::get_value_from_input_element;
use base64::Engine;
use log::*;
use shared::auth;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response, Storage};
use yew::{html, Component, Context, Html, InputEvent, SubmitEvent};

pub struct Login {
    username: String,
    password: String,
}

pub enum Message {
    Noop,
    Submit,
    UsernameUpdate(String),
    PasswordUpdate(String),
}

impl Component for Login {
    type Message = Message;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let username_onchange =
            ctx.link()
                .callback(|e: InputEvent| match get_value_from_input_element(e) {
                    Some(x) => Self::Message::UsernameUpdate(x),
                    None => Self::Message::Noop,
                });

        let password_onchange =
            ctx.link()
                .callback(|e: InputEvent| match get_value_from_input_element(e) {
                    Some(x) => Self::Message::PasswordUpdate(x),
                    None => Self::Message::Noop,
                });

        html! {
            <form
                onsubmit={ctx.link().callback(|e: SubmitEvent| {
                    e.prevent_default();
                    Self::Message::Submit
                })}
            >
                <div class="row">
                    <div class="col">
                        <label for="username">{ "Username" }</label>
                        <input name="username" type="text" oninput={username_onchange} />
                    </div>
                </div>
                <div class="row">
                    <div class="col">
                        <label for="password">{ "Password" }</label>
                        <input name="password" type="password" oninput={password_onchange} />
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Noop => false,
            Message::Submit => {
                debug!(
                    "TODO JEFF submit, username = {}, password = {}",
                    self.username, self.password
                );
                let username = self.username.clone();
                let password = self.password.clone();
                ctx.link().send_future(async move {
                    let response = login(&username, &password).await;
                    debug!("TODO login complete, should be sending a success or failure message");
                    Message::Noop
                });
                false
            }
            Message::UsernameUpdate(value) => {
                self.username = value;
                false
            }
            Message::PasswordUpdate(value) => {
                self.password = value;
                false
            }
        }
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
