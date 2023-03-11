mod dom_utils;
mod http;
mod js_utils;
mod login;
mod users;

use std::error::Error;

use log::*;
use login::Login;
use users::UsersList;
use yew::{function_component, html, use_state, Callback, Html};

use crate::{
    http::{is_logged_in, logout},
    js_utils::js_value_to_string,
};

#[derive(Debug, PartialEq)]
enum State {
    LoggedIn,
    LoggedOut,
    Error(String),
}

#[function_component]
fn App() -> Html {
    let state = use_state(|| match is_logged_in() {
        Ok(true) => State::LoggedIn,
        Ok(false) => State::LoggedOut,
        Err(e) => State::Error(match e.as_string() {
            Some(s) => s,
            None => format!("{e:?}"),
        }),
    });

    let login_callback = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(State::LoggedIn);
        })
    };

    let logout_callback = {
        let state = state.clone();
        Callback::from(move |_| {
            if let Err(e) = logout() {
                error!("error logging out {}", js_value_to_string(e));
                // TODO display error message on screen
            }
            state.set(State::LoggedOut);
        })
    };

    match &*state {
        State::LoggedIn => html! {
            <div class="row">
                <div class="col"/>
                <div class="col-8">
                    <UsersList />

                    // TODO delete me
                    <button onclick={logout_callback}>{ "Log Out" }</button>
                </div>
                <div class="col"/>
            </div>
        },
        State::LoggedOut => html! {
            <div class="row">
                <div class="col"/>
                <div class="col-8">
                    <Login login_success={login_callback} />
                </div>
                <div class="col"/>
            </div>
        },
        State::Error(e) => html! {
            <div>{ e }</div>
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| format!("error configuring logging: {e:?}"))?;

    yew::Renderer::<App>::new().render();

    Ok(())
}
