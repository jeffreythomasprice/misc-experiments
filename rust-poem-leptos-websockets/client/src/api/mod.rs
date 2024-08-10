pub mod users;
pub mod websockets;

use std::fmt::Debug;

use anyhow::Result;
use leptos::{create_effect, create_rw_signal, RwSignal, SignalGet, SignalGetUntracked, SignalSet};
use log::*;
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use shared::ErrorResponse;

use crate::storage::{get_local_storage_string, set_local_storage_string};

const AUTH_TOKEN_STORANGE_NAME: &str = "auth_token";

#[derive(Clone)]
pub struct APIService {
    base_url: String,
    pub auth_token: RwSignal<Option<String>>,
}

impl APIService {
    pub fn new(base_url: String) -> Self {
        let auth_token = create_rw_signal(get_local_storage_string(AUTH_TOKEN_STORANGE_NAME));

        create_effect(move |_| {
            set_local_storage_string(AUTH_TOKEN_STORANGE_NAME, auth_token.get())
        });

        APIService {
            base_url,
            auth_token,
        }
    }

    async fn get_json_response<ResponseType>(&self, path: &str) -> Result<ResponseType>
    where
        ResponseType: DeserializeOwned + Debug,
    {
        let url = self.join_url(path);
        trace!("making GET request to {}", url);
        let response = self
            .auth_header(reqwest::Client::new().get(url.clone()))
            .send()
            .await?;
        self.json_response("GET", &url, response).await
    }

    async fn post_json_request_json_response<RequestType, ResponseType>(
        &self,
        path: &str,
        request: &RequestType,
    ) -> Result<ResponseType>
    where
        RequestType: Serialize + Debug,
        ResponseType: DeserializeOwned + Debug,
    {
        let url = self.join_url(path);
        trace!("making POST request to {}, body={:?}", url, request);
        let response = self
            .auth_header(reqwest::Client::new().post(url.clone()))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(request)?)
            .send()
            .await?;
        self.json_response("POST", &url, response).await
    }

    fn join_url(&self, path: &str) -> String {
        if path.starts_with("/") {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    async fn json_response<ResponseType>(
        &self,
        method: &str,
        url: &str,
        response: Response,
    ) -> Result<ResponseType>
    where
        ResponseType: DeserializeOwned + Debug,
    {
        let status = response.status();
        let response_str = format!("status={}, headers={:?}", status, response.headers());
        let response_body = response.bytes().await?;
        if status.is_success() {
            let response_body: ResponseType = serde_json::from_slice(&response_body)?;
            debug!(
                "{} {} response: {}, body={:?}",
                method, url, response_str, response_body
            );
            Ok(response_body)
        } else {
            if status == StatusCode::UNAUTHORIZED {
                self.auth_token.set(None);
            }
            let response_body: ErrorResponse = serde_json::from_slice(&response_body)?;
            error!(
                "{} {} response: {}, body={:?}",
                method, url, response_str, response_body
            );
            Err(anyhow::Error::msg(response_body.message))
        }
    }

    fn auth_header(&self, r: RequestBuilder) -> RequestBuilder {
        if let Some(auth_token) = self.auth_token.get_untracked() {
            r.bearer_auth(auth_token)
        } else {
            r
        }
    }
}
