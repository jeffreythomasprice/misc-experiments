mod dom_utils;
mod login;
mod users;

use std::error::Error;

use log::*;
use login::{get_authorization_header, Login};
use users::UsersList;
use yew::{function_component, html, use_state, Callback, Component, Context, Html};

#[function_component]
fn App() -> Html {
    let is_logged_in = use_state(|| {
        // TODO error handling
        get_authorization_header().unwrap().is_some()
    });
    let login_success = {
        let is_logged_in = is_logged_in.clone();
        Callback::from(move |_| {
            is_logged_in.set(true);
        })
    };
    html! {
        <div>
            if *is_logged_in {
                <div class="row">
                    <div class="col"/>
                    <div class="col-10">
                        <UsersList />
                    </div>
                    <div class="col"/>
                </div>
            } else {
                <div class="row">
                    <div class="col"/>
                    <div class="col-8">
                        <Login login_success={login_success} />
                    </div>
                    <div class="col"/>
                </div>
            }
        </div>
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| format!("error configuring logging: {e:?}"))?;

    yew::Renderer::<App>::new().render();

    Ok(())
}
