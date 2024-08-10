use std::sync::Arc;

use leptos::{
    component, create_action, create_signal, event_target_value, expect_context, view, IntoView,
    RwSignal, SignalGet, SignalSet, WriteSignal,
};
use leptos_router::A;
use shared::{LogInRequest, LogInResponse};

use crate::api::APIService;

#[component]
#[allow(non_snake_case)]
pub fn Login() -> impl IntoView {
    let api_service = expect_context::<APIService>();

    let (username, set_username) = create_signal("".to_owned());
    let (password, set_password) = create_signal("".to_owned());

    let (error_message, set_error_message) = create_signal::<Option<String>>(None);

    // TODO use a <Suspense> tag
    // TODO disable button while loading, or is that what suspense does?

    let log_in_action = create_action(move |request: &LogInRequest| {
        let request = request.clone();
        set_error_message.set(None);
        {
            let api_service = api_service.clone();
            async move {
                if let Err(e) = api_service.log_in(&request).await {
                    set_error_message.set(Some(e.to_string()));
                }
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
                                    autofocus
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
