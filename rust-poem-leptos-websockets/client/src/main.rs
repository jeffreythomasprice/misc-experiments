mod api;
mod components;
mod constants;
mod storage;

use anyhow::{anyhow, Result};
use api::APIService;
use components::{ForgotPassword, Home, Login, Messages, Nav, NavItem, SignUp};
use constants::BASE_URL;
use leptos::*;
use leptos_router::{ProtectedRoute, Redirect, Route, Router, Routes};
use log::Level;
use std::panic;

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    provide_context(APIService::new(BASE_URL.to_owned()));

    let is_authenticated = {
        let api_service = expect_context::<APIService>();
        create_memo(move |_| api_service.auth_token.get().is_some())
    };

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
