mod api;
mod components;
mod constants;

use anyhow::{anyhow, Result};
use api::APIService;
use components::{ForgotPassword, Home, Login, Messages, Nav, NavItem, SignUp};
use constants::BASE_URL;
use leptos::*;
use leptos_router::{NavigateOptions, ProtectedRoute, Redirect, Route, Router, Routes};
use log::Level;
use std::panic;

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // TODO load and store auth token from local storage

    provide_context(APIService::new(BASE_URL.to_owned()));

    let is_authenticated = {
        let api_service = expect_context::<APIService>();
        create_memo(move |_| api_service.auth_token.get().is_some())
    };

    create_effect(move |_| {
        if is_authenticated.get() {
            leptos_router::use_navigate()("/home", NavigateOptions::default());
        } else {
            leptos_router::use_navigate()("/login", NavigateOptions::default());
        }
    });

    mount_to_body(move || {
        view! {
            <Router>
                <Show when=move || { is_authenticated.get() }>
                    <Nav>
                        <NavItem href="/home".to_string()>Home</NavItem>
                        <NavItem href="/messages".to_string()>Messages</NavItem>
                    </Nav>
                </Show>
                <Routes>
                    <Route path="/login" view=Login/>
                    <Route path="/forgotPassword" view=ForgotPassword/>
                    <Route path="/signUp" view=SignUp/>

                    <ProtectedRoute
                        path="/home"
                        redirect_path="/login"
                        condition=move || is_authenticated.get()
                        view=Home
                    />
                    <ProtectedRoute
                        path="/messages"
                        redirect_path="/login"
                        condition=move || is_authenticated.get()
                        view=Messages
                    />
                    <Route path="/*any" view=|| view! { <Redirect path="/login"/> }/>
                </Routes>
            </Router>
        }
    });

    Ok(())
}
