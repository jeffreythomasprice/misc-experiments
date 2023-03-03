mod dom_utils;
mod login;
mod users;

use std::error::Error;

use log::*;
use login::{get_authorization_header, Login};
use users::UsersList;
use yew::{function_component, html, use_state, Callback, Html};

#[derive(Debug, PartialEq)]
enum State {
    LoggedIn,
    LoggedOut,
    Error(String),
}

#[function_component]
fn App() -> Html {
    let state = use_state(|| match get_authorization_header() {
        Ok(Some(_)) => State::LoggedIn,
        Ok(None) => State::LoggedOut,
        Err(e) => State::Error(match e.as_string() {
            Some(s) => s,
            None => format!("{e:?}"),
        }),
    });

    let login_success = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(State::LoggedIn);
        })
    };

    match &*state {
        State::LoggedIn => html! {
            <div class="row">
                <div class="col"/>
                <div class="col-8">
                    <UsersList />
                </div>
                <div class="col"/>
            </div>
        },
        State::LoggedOut => html! {
            <div class="row">
                <div class="col"/>
                <div class="col-8">
                    <Login login_success={login_success} />
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
