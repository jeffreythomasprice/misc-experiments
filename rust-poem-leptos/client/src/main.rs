use leptos::*;
use log::*;
use shared::ClicksResponse;

const BASE_URL: &str = "http://127.0.0.1:8001";

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(None);

    create_resource(
        || (),
        move |_| async move {
            async fn f() -> Result<ClicksResponse, reqwest::Error> {
                reqwest::get(format!("{BASE_URL}/click"))
                    .await?
                    .json::<ClicksResponse>()
                    .await
            }
            match f().await {
                Ok(response) => {
                    set_count(Some(response.clicks));
                }
                Err(e) => {
                    error!("error making request: {e:?}");
                }
            }
        },
    );

    let send_click_request = create_action(move |_: &()| async move {
        async fn f() -> Result<ClicksResponse, reqwest::Error> {
            reqwest::Client::new()
                .post(format!("{BASE_URL}/click"))
                .send()
                .await?
                .json::<ClicksResponse>()
                .await
        }
        match f().await {
            Ok(response) => {
                set_count(Some(response.clicks));
            }
            Err(e) => {
                error!("error making request: {e:?}");
            }
        }
    });

    let on_click = move |_| {
        send_click_request.dispatch(());
    };

    view! {
        {move || match count() {
            Some(count) => view! { <div>Clicks: {count}</div> },
            None => view! { <div>Loading...</div> },
        }}

        <button on:click=on_click>Click Me</button>
    }
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();

    mount_to_body(|| view! { <App/> })
}
