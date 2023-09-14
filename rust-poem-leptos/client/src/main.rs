use leptos::ev::KeyboardEvent;
use leptos::*;
use log::*;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::models::{ClientHelloRequest, ClientHelloResponse};
use shared::websockets::{Message, WebSocketChannel};
use std::cell::RefCell;
use std::panic;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsValue;

#[component]
fn Login<F>(cx: Scope, submit: F) -> impl IntoView
where
    F: Fn(ActiveClient) + 'static,
{
    let submit = Rc::new(submit);

    let http_client = use_context::<Arc<HttpClient>>(cx).unwrap();

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
                    submit(ActiveClient {
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

    provide_context(cx, Arc::new(HttpClient::new(BASE_URL.to_string())));

    let websocket_service = Arc::new(Mutex::new(RefCell::new(WebSocketService::new(BASE_URL))));
    provide_context(cx, websocket_service.clone());

    let (is_logged_in, set_logged_in) = create_signal(cx, false);
    let (name, set_name) = create_signal(cx, "".to_string());

    let login = create_action(cx, move |input: &ActiveClient| {
        let input = input.clone();
        let websocket_service = websocket_service.clone();
        async move {
            let websocket_service = websocket_service.lock().unwrap();
            websocket_service.borrow_mut().log_in(input.clone());
            set_name(input.name);
            set_logged_in(true);
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

    // TODO JEFF no
    spawn_local(async {
        if let Err(e) = test_websockets().await {
            log::error!("error doing test websockets: {e:?}");
        }
    });

    mount_to_body(|cx| {
        view! { cx,
            <App/>
        }
    })
}

struct HttpClient {
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn client_hello(
        &self,
        request: &ClientHelloRequest,
    ) -> Result<ClientHelloResponse, reqwest::Error> {
        self.make_json_request_json_response(Method::POST, "client", request)
            .await
    }

    async fn make_json_request_json_response<RequestType, ResponseType>(
        &self,
        method: Method,
        path: &str,
        request: &RequestType,
    ) -> Result<ResponseType, reqwest::Error>
    where
        RequestType: Serialize,
        ResponseType: DeserializeOwned,
    {
        let client = reqwest::Client::new();
        let response = client
            .request(method, format!("{}/{}", self.base_url, path))
            .json(request)
            .send()
            .await?;
        let response_body: ResponseType = response.json().await?;
        Ok(response_body)
    }
}

#[derive(Debug, Clone)]
struct ActiveClient {
    client_id: String,
    name: String,
}

struct WebSocketService {
    base_url: String,
    client: Option<ActiveClient>,
}

impl WebSocketService {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: None,
        }
    }

    pub fn log_in(&mut self, client: ActiveClient) {
        self.client.replace(client);

        /*
        TODO JEFF do websockets

        if we were already running:
            stop
        start a new websocket reader and writer future
        */
    }
}

async fn test_websockets() -> Result<(), JsValue> {
    let (sender, mut receiver) =
        shared::websockets::client::connect("ws://127.0.0.1:8001/ws")?.split();

    spawn_local(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                Ok(Message::Text(value)) => {
                    debug!("TODO JEFF got text message from server, {}", value)
                }
                Ok(Message::Binary(value)) => debug!(
                    "TODO JEFF got binary message from client, {} bytes",
                    value.len()
                ),
                Err(_e) => log::error!("TODO JEFF error from websocket"),
            }
        }
    });

    spawn_local(async move {
        if let Err(e) = sender
            .send(Message::Text(
                "TODO JEFF test message from client".to_string(),
            ))
            .await
        {
            log::error!("TODO JEFF error sending test message: {e:?}");
        }
    });

    Ok(())
}
