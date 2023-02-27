mod login;
mod rename;

use std::error::Error;

use log::*;
use login::Login;
use yew::{html, Component, Context, Html};

struct App;

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div class="row">
                    <div class="col"/>
                    <div class="col-8">
                        <Login />
                    </div>
                    <div class="col"/>
                </div>
            </div>
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    console_log::init_with_level(Level::Trace)
        .map_err(|e| format!("error configuring logging: {e:?}"))?;

    yew::Renderer::<App>::new().render();

    Ok(())
}
