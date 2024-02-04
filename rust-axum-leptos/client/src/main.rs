mod requests;

use std::panic;

use leptos::*;
use log::*;
use requests::http_request_json_body_json_response;

use shared::{LoginRequest, LoginResponse};

#[component]
fn LoginForm(#[prop(into)] on_login: Callback<LoginResponse>) -> impl IntoView {
    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());

    let (errors, set_errors) = create_signal::<Vec<String>>(Vec::new());

    let (request, set_request) = create_signal::<Option<LoginRequest>>(None);
    create_local_resource(request, move |request| async move {
        if let Some(request) = request {
            match login(&request).await {
                Ok(response) => {
                    set_errors(vec![]);
                    on_login(response);
                }
                Err(message) => {
                    set_errors(vec![message]);
                }
            }
        }
    });

    view! {
        <form
            class="login-form"
            on:submit=move |ev| {
                ev.prevent_default();
                set_request(
                    Some(LoginRequest {
                        username: username(),
                        password: password(),
                    }),
                );
            }
        >

            <div class="grid">
                <label for="username">Username:</label>
                <input
                    type="text"
                    name="username"
                    placeholder="Username"
                    prop:value=username
                    autofocus
                    on:input=move |ev| {
                        set_username(event_target_value(&ev));
                    }
                />

                <label for="password">Password:</label>
                <input
                    type="password"
                    name="password"
                    placeholder="Password"
                    prop:value=password
                    on:input=move |ev| {
                        set_password(event_target_value(&ev));
                    }
                />

                <div class="submit-button">
                    <button>Log In</button>
                </div>
            </div>
        </form>
        <div class="errors">
            {move || {
                errors().into_iter().map(|error| view! { <div>{error}</div> }).collect_view()
            }}

        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (login_status, set_login_status) = create_signal::<Option<LoginResponse>>(None);

    let on_login = move |result| {
        set_login_status(Some(result));
    };

    view! {
        <Show
            when=move || { login_status().is_some() }
            fallback=move || view! { <LoginForm on_login=on_login/> }
        >
            <div>
                {move || format!("TODO handle logged in case: {:?}", login_status().unwrap())}
            </div>
        </Show>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|| view! { <App/> })
}

async fn login(request: &LoginRequest) -> Result<LoginResponse, String> {
    match http_request_json_body_json_response("POST", "/login", request).await {
        Ok(result) => Ok(result),
        Err(requests::Error::BadStatusCode(401)) => Err("Invalid username or password".to_string()),
        _ => Err("An error occurred".to_string()),
    }
}
