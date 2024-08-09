pub mod users;

use std::fmt::Debug;

use anyhow::Result;
use log::*;
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};
use shared::ErrorResponse;

#[derive(Clone)]
pub struct APIService {
    base_url: String,
}

impl APIService {
    pub fn new(base_url: &str) -> Self {
        APIService {
            base_url: base_url.to_owned(),
        }
    }

    async fn get_json_response<ResponseType>(&self, path: &str) -> Result<ResponseType>
    where
        ResponseType: DeserializeOwned + Debug,
    {
        let url = self.join_url(path);
        trace!("making GET request to {}", url);
        let response = reqwest::get(url.clone()).await?;
        Ok(Self::json_response("GET", &url, response).await?)
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
        let client = reqwest::Client::new();
        let response = client
            .post(url.clone())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(request)?)
            .send()
            .await?;
        Ok(Self::json_response("POST", &url, response).await?)
    }

    fn join_url(&self, path: &str) -> String {
        if path.starts_with("/") {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    async fn json_response<ResponseType>(
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
            let response_body: ErrorResponse = serde_json::from_slice(&response_body)?;
            error!(
                "{} {} response: {}, body={:?}",
                method, url, response_str, response_body
            );
            Err(anyhow::Error::msg(response_body.message))
        }
    }
}
