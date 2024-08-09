use leptos::{component, view, Children, IntoView};
use leptos_router::A;

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
            <ul class="flex border-b">{children()}</ul>
        </div>
    }
}
