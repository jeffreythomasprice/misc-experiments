mod api;
mod constants;
mod websockets;

use anyhow::{anyhow, Result};
use api::APIService;
use chrono::{DateTime, Utc};
use constants::{BASE_URL, WS_URL};
use futures::{Sink, SinkExt, StreamExt};
use leptos::*;
use leptos_router::{Redirect, Route, Router, Routes, A};
use log::Level;
use log::*;
use shared::{
    CreateUserRequest, LogInRequest, WebsocketClientToServerMessage, WebsocketServerToClientMessage,
};
use std::{
    panic,
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use websockets::new_websocket_with_url;

// TODO move all the components to new files

#[derive(Debug, Clone)]
struct DisplayedMessage {
    id: Uuid,
    received_timestamp: DateTime<Utc>,
    message: String,
}

#[component]
#[allow(non_snake_case)]
fn Messages(
    messages: ReadSignal<Vec<DisplayedMessage>>,
    on_submit: impl Fn(String) + 'static,
) -> impl IntoView {
    let (next_message, set_next_message) = create_signal("".to_owned());

    view! {
        <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
            <form on:submit=move |e| {
                e.prevent_default();
                on_submit(next_message.get());
                set_next_message.set("".to_owned());
            }>
                <input
                    type="text"
                    placeholder="Message"
                    name="message"
                    class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                    prop:value=next_message
                    on:input=move |e| set_next_message.set(event_target_value(&e))
                />
            </form>
            <For
                each=move || { messages.get() }
                key=|msg| { msg.id }
                children=move |msg| {
                    view! {
                        <div>
                            {format!("{}: {}", msg.received_timestamp.to_rfc3339(), msg.message)}
                        </div>
                    }
                }
            />

        </div>
    }
}

#[component]
#[allow(non_snake_case)]
fn LoginForm(api_service: APIService) -> impl IntoView {
    let (username, set_username) = create_signal("".to_owned());
    let (password, set_password) = create_signal("".to_owned());

    let (error_message, set_error_message) = create_signal::<Option<String>>(None);

    let log_in_action = create_action(move |request: &LogInRequest| {
        let request = request.clone();
        set_error_message.set(None);
        {
            let api_service = api_service.clone();
            async move {
                match api_service.log_in(&request).await {
                    Ok(response) => {
                        // TODO pass this up the chain
                        debug!("TODO login response: {response:?}");
                    }
                    Err(e) => {
                        set_error_message.set(Some(e.to_string()));
                    }
                };
            }
        }
    });

    view! {
        <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-sm">
                <h2 class="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
                    Sign in to your account
                </h2>
            </div>

            <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
                <form
                    class="space-y-6"
                    on:submit=move |e| {
                        e.prevent_default();
                        log_in_action
                            .dispatch(LogInRequest {
                                username: username.get(),
                                password: password.get(),
                            });
                    }
                >

                    <div>
                        <div>
                            <label
                                for="email"
                                class="block text-sm font-medium leading-6 text-gray-900"
                            >
                                Email address
                            </label>
                            <div class="mt-2">
                                <input
                                    name="username"
                                    type="text"
                                    placeholder="Username"
                                    class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                                    prop:value=username
                                    on:input=move |e| set_username.set(event_target_value(&e))
                                />
                            </div>
                        </div>
                    </div>

                    <div>
                        <label
                            for="password"
                            class="block text-sm font-medium leading-6 text-gray-900"
                        >
                            Password
                        </label>
                        <div class="mt-2">
                            <input
                                name="password"
                                type="password"
                                placeholder="Password"
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                                prop:value=password
                                on:input=move |e| set_password.set(event_target_value(&e))
                            />
                            <div class="flex justify-end text-sm">
                                <A
                                    href="/forgotPassword"
                                    class="font-semibold text-indigo-600 hover:text-indigo-500"
                                >
                                    Forgot password?
                                </A>
                            </div>
                        </div>
                    </div>

                    {move || {
                        error_message
                            .get()
                            .map(|s| {
                                view! {
                                    <div
                                        class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative"
                                        role="alert"
                                    >
                                        <span class="block sm:inline">{s}</span>
                                    </div>
                                }
                            })
                    }}

                    <div>
                        <button
                            type="submit"
                            class="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                        >
                            Log In
                        </button>
                    </div>

                    <div>
                        <A
                            href="/signUp"
                            class="flex w-full justify-center rounded-md bg-gray-200 px-3 py-1.5 text-sm font-semibold leading-6 text-white-800 shadow-sm hover:bg-gray-300 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-300"
                        >
                            Sign Up
                        </A>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
#[allow(non_snake_case)]
fn ForgotPassword() -> impl IntoView {
    view! {
        <div>
            Placeholder, probably not going to actually implement emailing the user a password reset code and all that nonsense.
        </div>
    }
}

#[component]
#[allow(non_snake_case)]
fn SignUp(api_service: APIService) -> impl IntoView {
    let (username, set_username) = create_signal("".to_owned());
    let (password, set_password) = create_signal("".to_owned());
    let (confirm_password, set_confirm_password) = create_signal("".to_owned());

    let create_user_action = create_action(move |request: &CreateUserRequest| {
        // TODO check password and confirm equals

        let request = request.clone();
        {
            let api_service = api_service.clone();
            async move {
                match api_service.create_user(&request).await {
                    Ok(response) => {
                        // TODO pass this up the chain
                        debug!("TODO create user response: {response:?}");
                    }
                    Err(e) => {
                        // TODO put error message on screen
                        error!("error logging in: {e:?}");
                    }
                };
            }
        }
    });

    view! {
        <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-sm">
                <h2 class="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
                    Create User
                </h2>
            </div>

            <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
                <form
                    class="space-y-6"
                    on:submit=move |e| {
                        e.prevent_default();
                        create_user_action
                            .dispatch(CreateUserRequest {
                                username: username.get(),
                                password: password.get(),
                            });
                    }
                >

                    <div>
                        <div>
                            <label
                                for="email"
                                class="block text-sm font-medium leading-6 text-gray-900"
                            >
                                Email address
                            </label>
                            <div class="mt-2">
                                <input
                                    name="username"
                                    type="text"
                                    placeholder="Username"
                                    class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                                    prop:value=username
                                    on:input=move |e| set_username.set(event_target_value(&e))
                                />
                            </div>
                        </div>
                    </div>

                    <div>
                        <label
                            for="password"
                            class="block text-sm font-medium leading-6 text-gray-900"
                        >
                            Password
                        </label>
                        <div class="mt-2">
                            <input
                                name="password"
                                type="password"
                                placeholder="Password"
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                                prop:value=password
                                on:input=move |e| set_password.set(event_target_value(&e))
                            />
                        </div>
                    </div>

                    <div>
                        <label
                            for="password"
                            class="block text-sm font-medium leading-6 text-gray-900"
                        >
                            Confirm Password
                        </label>
                        <div class="mt-2">
                            <input
                                name="password"
                                type="password"
                                placeholder="Password"
                                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                                prop:value=confirm_password
                                on:input=move |e| set_confirm_password.set(event_target_value(&e))
                            />
                        </div>
                    </div>

                    <div>
                        <button
                            type="submit"
                            class="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                        >
                            Log In
                        </button>
                    </div>

                    <div>
                        <A
                            href="/login"
                            class="flex w-full justify-center rounded-md bg-gray-200 px-3 py-1.5 text-sm font-semibold leading-6 text-white-800 shadow-sm hover:bg-gray-300 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-300"
                        >
                            Cancel
                        </A>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
#[allow(non_snake_case)]
fn NavItem(href: String, children: Children) -> impl IntoView {
    view! {
        <li class="mb-px mr-1 text-gray-500">
            <A
                active_class="text-blue-700 border-l border-t border-r rounded-t"
                class="bg-white inline-block py-2 px-4 font-semibold"
                href=href
            >
                {children()}
            </A>
        </li>
    }
}

#[component]
#[allow(non_snake_case)]
fn Nav(children: Children) -> impl IntoView {
    view! {
        <div class="rounded-t-lg overflow-hidden border-t border-l border-r border-gray-400 p-4">
            <ul class="flex border-b">{children()}</ul>
        </div>
    }
}

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let api_service = APIService::new(BASE_URL);

    let (messages, set_messages) = create_signal(Vec::new());

    let websocket_sink = Arc::new(Mutex::new(None));

    {
        let websocket_sink = websocket_sink.clone();
        spawn_local(async move {
            match websocket_demo(move |msg| {
                let WebsocketServerToClientMessage::Message(msg) = msg;
                set_messages.update(|messages| {
                    messages.push(DisplayedMessage {
                        id: Uuid::new_v4(),
                        received_timestamp: Utc::now(),
                        message: msg,
                    })
                });
            })
            .await
            {
                Ok(sink) => {
                    let mut websocket_sink = websocket_sink.lock().unwrap();
                    websocket_sink.replace(sink);
                }
                Err(e) => error!("error running websocket demo: {e:?}"),
            };
        });
    }

    mount_to_body(move || {
        view! {
            <Router>
                <Nav>
                    <NavItem href="/messages".to_string()>Messages</NavItem>
                    <NavItem href="/login".to_string()>Login</NavItem>
                </Nav>
                <Routes>
                    <Route
                        path="/messages"
                        view=move || {
                            view! {
                                <Messages
                                    messages=messages
                                    on_submit={
                                        let websocket_sink = websocket_sink.clone();
                                        move |msg| {
                                            info!("submitting outgoing message: {msg}");
                                            let websocket_sink = websocket_sink.clone();
                                            spawn_local(async move {
                                                if let Some(sink) = &mut *websocket_sink.lock().unwrap() {
                                                    if let Err(e) = sink
                                                        .send(WebsocketClientToServerMessage::Message(msg))
                                                        .await
                                                    {
                                                        error!("error sending to websocket: {e:?}");
                                                    }
                                                }
                                            });
                                        }
                                    }
                                />
                            }
                        }
                    />

                    <Route
                        path="/login"
                        view={
                            let api_service = api_service.clone();
                            move || view! { <LoginForm api_service=api_service.clone()/> }
                        }
                    />

                    <Route path="/forgotPassword" view=move || view! { <ForgotPassword/> }/>

                    <Route
                        path="/signUp"
                        view={
                            let api_service = api_service.clone();
                            move || view! { <SignUp api_service=api_service.clone()/> }
                        }
                    />

                    // TODO page to see when logged in with logout button

                    <Route path="/*any" view=|| view! { <Redirect path="/login"/> }/>
                </Routes>
            </Router>
        }
    });

    Ok(())
}

// TODO rename me? move me?
async fn websocket_demo(
    output: impl Fn(WebsocketServerToClientMessage) + 'static,
) -> Result<
    std::pin::Pin<Box<dyn Sink<WebsocketClientToServerMessage, Error = ws_stream_wasm::WsErr>>>,
> {
    let (sink, mut stream) = new_websocket_with_url::<
        WebsocketClientToServerMessage,
        WebsocketServerToClientMessage,
    >(WS_URL)
    .await?;

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            info!("received message from websocket: {msg:?}");
            output(msg);
        }
    });

    Ok(sink)
}
