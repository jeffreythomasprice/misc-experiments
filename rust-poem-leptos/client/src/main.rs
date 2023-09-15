use leptos::ev::KeyboardEvent;
use leptos::*;
use log::*;

use shared::models::ClientHelloRequest;

use std::panic;
use std::rc::Rc;

mod clients;

#[component]
fn Login<F>(cx: Scope, submit: F) -> impl IntoView
where
    F: Fn(clients::websocket::LogInRequest) + 'static,
{
    let submit = Rc::new(submit);

    let http_client = use_context::<clients::http::Client>(cx).unwrap();

    let (value, set_value) = create_signal(cx, "".to_string());

    let on_submit = Rc::new(move || {
        let http_client = http_client.clone();
        let submit = submit.clone();
        let name = value();
        spawn_local(async move {
            match http_client
                .client_hello(&ClientHelloRequest { name: name.clone() })
                .await
            {
                Ok(response) => {
                    submit(clients::websocket::LogInRequest {
                        client_id: response.client_id,
                        name,
                    });
                }
                Err(e) => {
                    log::warn!("error making client hello request: {e:?}");
                }
            }
        });
    });

    let on_button_click = {
        let on_submit = on_submit.clone();
        move |_| {
            on_submit();
        }
    };

    let on_input_keyup = move |e: KeyboardEvent| {
        if e.key() == "Enter" {
            on_submit();
        }
    };

    let on_input_input = move |e| {
        set_value(event_target_value(&e));
    };

    view! { cx,
        <div>
            <input type="text" autofocus on:keyup=on_input_keyup on:input=on_input_input />
            <button on:click=on_button_click>Start</button>
        </div>
    }
}

#[component]
fn LoggedIn(cx: Scope, name: String) -> impl IntoView {
    view! { cx,
        <div>{name}</div>
    }
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    const BASE_URL: &str = "http://localhost:8001";

    provide_context(cx, clients::http::Client::new(BASE_URL.to_string()));

    let websocket_client = clients::websocket::Client::new(BASE_URL);

    let (is_logged_in, set_logged_in) = create_signal(cx, false);
    let (name, set_name) = create_signal(cx, "".to_string());

    let login = create_action(cx, move |input: &clients::websocket::LogInRequest| {
        let input = input.clone();
        let ws = websocket_client.clone();
        async move {
            if let Err(e) = ws.log_in(input.clone()) {
                log::error!("error logging in {e:?}");
            } else {
                set_name(input.name);
                set_logged_in(true);
            }
        }
    });

    let content = move || {
        if is_logged_in() {
            view! { cx,
                <LoggedIn name={name()}/>
            }
        } else {
            view! { cx,
                <Login submit=move |client| {
                    login.dispatch(client);
                }/>
            }
        }
    };

    view! { cx,
        <>{content}</>
    }
}

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|cx| {
        view! { cx,
            <App/>
        }
    })
}
