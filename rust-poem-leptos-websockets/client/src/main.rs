mod api;
mod components;
mod constants;

use anyhow::{anyhow, Result};
use api::websockets::WebsocketService;
use api::APIService;
use chrono::{DateTime, Utc};
use components::{ForgotPassword, Login, Messages, Nav, NavItem, SignUp};
use constants::{BASE_URL, WS_URL};
use futures::{Sink, SinkExt, StreamExt};
use leptos::*;
use leptos_router::{Redirect, Route, Router, Routes};
use log::Level;
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use std::pin::Pin;
use std::{
    panic,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).map_err(|e| anyhow!("{e:?}"))?;
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let api_service = APIService::new(BASE_URL.to_owned());

    let websocket_service = WebsocketService::new(WS_URL.to_owned());

    mount_to_body(move || {
        view! {
            <Router>
                <Nav>
                    <NavItem href="/messages".to_string()>Messages</NavItem>
                    <NavItem href="/login".to_string()>Login</NavItem>
                </Nav>
                <Routes>
                    <Route
                        path="/messages"
                        view={
                            let websocket_service = websocket_service.clone();
                            move || {
                                view! { <Messages service=websocket_service.clone()/> }
                            }
                        }
                    />

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
