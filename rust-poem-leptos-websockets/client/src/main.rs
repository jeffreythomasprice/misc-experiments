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
use std::{panic, sync::Arc};

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // TODO load and store auth token from local storage
    // TODO initial page based on whether we're logged in

    let api_service = Arc::new(APIService::new(BASE_URL.to_owned()));

    let is_authenticated = {
        let api_service = api_service.clone();
        create_memo(move |_| api_service.auth_token().get().is_some())
    };

    create_effect(move |_| {
        if is_authenticated.get() {
            leptos_router::use_navigate()("/home", NavigateOptions::default());
        }
    });

    mount_to_body(move || {
        view! {
            <Router>

                {
                    let is_authenticated = is_authenticated.clone();
                    view! {
                        <Show when=move || is_authenticated.get()>
                            <Nav>
                                <NavItem href="/home".to_string()>Home</NavItem>
                                <NavItem href="/messages".to_string()>Messages</NavItem>
                            </Nav>
                        </Show>
                    }
                }
                <Routes>
                    <Route path="/messages" view=|| view! { <Messages/> }/>

                    <Route
                        path="/login"
                        view={
                            let api_service = api_service.clone();
                            move || {
                                view! { <Login api_service=api_service.clone()/> }
                            }
                        }
                    />

                    <Route path="/forgotPassword" view=ForgotPassword/>

                    <Route
                        path="/signUp"
                        view={
                            let api_service = api_service.clone();
                            move || view! { <SignUp api_service=api_service.clone()/> }
                        }
                    />

                    <ProtectedRoute
                        path="/home"
                        redirect_path="/login"
                        condition=move || is_authenticated.get()
                        view=Home
                    />

                    <Route path="/*any" view=|| view! { <Redirect path="/login"/> }/>
                </Routes>
            </Router>
        }
    });

    Ok(())
}
