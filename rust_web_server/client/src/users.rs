use log::*;
use shared::user::UserResponse;
use yew::{function_component, html, platform::spawn_local, use_state, Html};

use crate::{
    http::{self, get_users},
    js_utils::js_value_to_string,
};

enum State {
    Fetching,
    Error(String),
    Ready(Vec<UserResponse>),
}

#[function_component]
pub fn UsersList() -> Html {
    let state = use_state(|| State::Fetching);

    match &*state {
        State::Fetching => {
            let state = state.clone();
            spawn_local(async move {
                match get_users().await {
                    Ok(users) => {
                        trace!("got users");
                        state.set(State::Ready(users));
                    }
                    Err(e) => {
                        // TODO common behvaior for all api calls to turn 401 into a navigate to login page
                        error!("error getting users: {e:?}");
                        state.set(State::Error(format!("{e:?}")));
                    }
                }
            });
        }
        _ => {}
    }

    match &*state {
        State::Fetching => html! {
            <div class="row">
                <p>{ "Loading..." }</p>
            </div>
        },
        State::Error(e) => html! {
            <div class="row">
                <p class="text-error">{ e }</p>
            </div>
        },
        State::Ready(users) => html! {
            {
                users.iter().map(|user| {
                    if user.is_admin {
                        html! {
                            <p>{ &user.name }{ " (admin)" }</p>
                        }
                    } else {
                        html! {
                            <p>{ &user.name }</p>
                        }
                    }
                }).collect::<Html>()
            }
        },
    }
}
