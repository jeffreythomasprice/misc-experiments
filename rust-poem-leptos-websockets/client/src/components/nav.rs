use leptos::{component, expect_context, view, Children, IntoView, SignalSet};
use leptos_router::{NavigateOptions, A};

use crate::api::APIService;

#[component]
#[allow(non_snake_case)]
pub fn NavItem(href: String, children: Children) -> impl IntoView {
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
pub fn Nav(children: Children) -> impl IntoView {
    view! {
        <div class="rounded-t-lg overflow-hidden border-t border-l border-r border-gray-400 p-4">
            <ul class="flex border-b">{children()} <LogoutButton/></ul>
        </div>
    }
}

#[component]
#[allow(non_snake_case)]
pub fn LogoutButton() -> impl IntoView {
    let api_service = expect_context::<APIService>();

    view! {
        <li class="ml-auto">
            <button
                class="bg-transparent hover:bg-blue-500 text-blue-700 font-semibold hover:text-white py-2 px-4 border border-blue-500 hover:border-transparent rounded"
                on:click=move |_| {
                    api_service.auth_token.set(None);
                    leptos_router::use_navigate()("/login", NavigateOptions::default());
                }
            >

                Log Out
            </button>
        </li>
    }
}
