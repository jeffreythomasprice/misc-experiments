mod api;
mod components;
mod constants;

use anyhow::{anyhow, Result};
use api::APIService;
use components::{ForgotPassword, Login, Messages, Nav, NavItem, SignUp};
use constants::BASE_URL;
use leptos::*;
use leptos_router::{Redirect, Route, Router, Routes};
use log::Level;
use log::*;
use std::panic;

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let api_service = APIService::new(BASE_URL.to_owned());

    mount_to_body(move || {
        view! {
            <Router>
                <Nav>
                    <NavItem href="/messages".to_string()>Messages</NavItem>
                    <NavItem href="/login".to_string()>Login</NavItem>
                </Nav>
                <Routes>
                    <Route path="/messages" view=|| view! { <Messages/> }/>

                    <Route
                        path="/login"
                        view={
                            let api_service = api_service.clone();
                            move || view! { <Login api_service=api_service.clone()/> }
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
