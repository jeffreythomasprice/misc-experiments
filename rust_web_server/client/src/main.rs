mod rename;

use std::error::Error;

use log::*;
use rename::get_value_from_input_element;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlInputElement};
use yew::{html, use_node_ref, Component, Context, Html, InputEvent, SubmitEvent};

struct App;

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div class="row">
                    <div class="col"/>
                    <div class="col-8">
                        <Login />
                    </div>
                    <div class="col"/>
                </div>
            </div>
        }
    }
}

struct Login {
    username: String,
    password: String,
}

enum LoginMessage {
    Noop,
    Submit,
    UsernameUpdate(String),
    PasswordUpdate(String),
}

impl Component for Login {
    type Message = LoginMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
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
            LoginMessage::Noop => {}
            LoginMessage::Submit => {
                debug!(
                    "TODO JEFF submit, username = {}, password = {}",
                    self.username, self.password
                );
            }
            LoginMessage::UsernameUpdate(value) => {
                self.username = value;
            }
            LoginMessage::PasswordUpdate(value) => {
                self.password = value;
            }
        };
        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| format!("error configuring logging: {e:?}"))?;

    yew::Renderer::<App>::new().render();

    Ok(())
}
