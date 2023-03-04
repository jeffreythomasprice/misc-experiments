use crate::{
    dom_utils::get_value_from_input_element_event, http::login, js_utils::js_value_to_string,
};
use log::*;
use yew::{
    function_component, html, platform::spawn_local, use_state, Callback, Html, InputEvent,
    Properties, SubmitEvent,
};

#[derive(PartialEq, Properties)]
pub struct LoginProps {
    pub login_success: Callback<()>,
}

#[function_component]
pub fn Login(props: &LoginProps) -> Html {
    let username = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let error_message = use_state(|| -> Option<String> { None });

    let username_changed = {
        let state = username.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(x) = get_value_from_input_element_event(e) {
                state.set(x)
            }
        })
    };

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
        let error_message = error_message.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            trace!("logging in {}", *username);

            let login_success = login_success.clone();
            let username = username.clone();
            let password = password.clone();
            let error_message = error_message.clone();
            spawn_local(async move {
                match login(&username, &password).await {
                    Ok(_) => {
                        trace!("login success");
                        login_success.emit(());
                    }
                    Err(e) => {
                        // TODO interpret errors as bad credentials or other, get human-readable error message
                        error!("error logging in: {e:?}");
                        error_message.set(Some(format!("{e:?}")));
                    }
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
            if let Some(s) = &*error_message {
                <div class="row">
                    <p class="text-error">{ s }</p>
                </div>
            }
        </form>
    }
}
