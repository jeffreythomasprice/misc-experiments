use leptos::*;
use log::*;
use shared::{LoginRequest, LoginResponse};

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

    let do_login = move || {
        create_action(|input: &(String, String)| {
            let (username, password) = input.clone();
            login(username, password)
        })
        .dispatch((username(), password()));
    };

    view! {
        <form
            class="loginForm"
            on:submit=move |ev| {
                ev.prevent_default();
                do_login();
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
    view! { <LoginForm/> }
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
    // TODO check if it's an error response
    Ok(response.json::<LoginResponse>().await?)
}
