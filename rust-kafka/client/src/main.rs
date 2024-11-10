use std::panic;

use leptos::*;
use log::*;

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Trace).unwrap();

    mount_to_body(|| {
        view! { <Counter /> }
    })
}

#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div>Clicks: {count}</div>
        <button on:click=move |_| {
            let new_count = count() + 1;
            set_count(new_count);
            info!("count is now {}", new_count);
        }>Click Me</button>
    }
}
