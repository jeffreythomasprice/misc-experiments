use std::borrow::Borrow;

use leptos::*;
use log::*;
use shared::{LoginRequest, LoginResponse};

#[derive(Debug)]
struct ErrorResponse(shared::ErrorResponse);

impl From<reqwest::Error> for ErrorResponse {
    fn from(value: reqwest::Error) -> Self {
        Self(shared::ErrorResponse {
            messages: vec![format!("{value:?}")],
        })
    }
}

fn LoginForm() -> impl IntoView {
    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());

    let set_user = use_context::<WriteSignal<Option<LoginResponse>>>().unwrap();

    let login_action = create_action(move |input: &(String, String)| {
        let (username, password) = input.clone();
        async move {
            match login(username, password).await {
                Ok(response) => {
                    trace!("login successful: {response:?}");
                    set_user(Some(response));
                }
                Err(e) => error!("TODO login failed: {:?}", e.0.messages),
            }
        }
    });

    view! {
        <form
            class="loginForm"
            on:submit=move |ev| {
                ev.prevent_default();
                login_action.dispatch((username(), password()));
            }
        >

            <label for="username">Username</label>
            <input
                type="text"
                name="username"
                placeholder="Username"
                on:input=move |ev| { set_username(event_target_value(&ev)) }
                prop:value=username
            />
            <label for="password">Password</label>
            <input
                type="password"
                name="password"
                placeholder="Password"
                on:input=move |ev| { set_password(event_target_value(&ev)) }
                prop:value=password
            />
            <div class="buttons">
                <button type="submit">Login</button>
            </div>
        </form>
    }
}

#[component]
fn App() -> impl IntoView {
    let (user, set_user) = create_signal::<Option<LoginResponse>>(None);
    provide_context(set_user);

    view! {
        <LoginForm/>
        <Show when=move || { user().is_some() }>
            <div>"Hello, " {user().unwrap().username}</div>
        </Show>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    mount_to_body(|| App())
}

async fn login(username: String, password: String) -> Result<LoginResponse, ErrorResponse> {
    let request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    debug!("logging in, username = {}", username);
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8001/login")
        .json(&request)
        .send()
        .await?;
    trace!("login response = {response:?}");
    if response.status().is_success() {
        Ok(response.json::<LoginResponse>().await?)
    } else {
        Err(ErrorResponse(
            response.json::<shared::ErrorResponse>().await?,
        ))
    }
}
