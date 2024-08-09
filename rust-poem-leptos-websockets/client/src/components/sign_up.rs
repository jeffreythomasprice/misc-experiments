use leptos::{
    component, create_action, create_signal, event_target_value, view, IntoView, SignalGet,
    SignalSet,
};
use leptos_router::A;
use log::*;
use shared::CreateUserRequest;

use crate::api::APIService;

#[component]
#[allow(non_snake_case)]
pub fn SignUp(api_service: APIService) -> impl IntoView {
    let (username, set_username) = create_signal("".to_owned());
    let (password, set_password) = create_signal("".to_owned());
    let (confirm_password, set_confirm_password) = create_signal("".to_owned());

    // TODO use a <Suspense> tag
    // TODO disable button while loading, or is that what suspense does?

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
