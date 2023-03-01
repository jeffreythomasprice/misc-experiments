use yew::{function_component, html, platform::spawn_local, Html};

#[function_component]
pub fn UsersList() -> Html {
    spawn_local(async {
        todo!("fetch users");
    });

    html! {
        <div class="row">
            { "TODO show users" }
        </div>
    }
}
