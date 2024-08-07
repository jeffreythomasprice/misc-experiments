use super::APIService;

use anyhow::Result;
use log::*;
use shared::{LogInRequest, UserResponse};

impl APIService {
    pub async fn list_users(&self) -> Result<Vec<UserResponse>> {
        let response = reqwest::get(format!("{}/users", self.base_url)).await?;
        debug!("list users response: {:?}", response);
        let response_body = response.bytes().await?;
        let response_body = serde_json::from_slice(&response_body)?;
        debug!("list users response body: {:?}", response_body);
        Ok(response_body)
    }

    pub async fn log_in(&self, request: &LogInRequest) -> Result<UserResponse> {
        // TODO deduplicate this stuff with other requests
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/login", self.base_url))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(request)?)
            .send()
            .await?;
        debug!("login response: {:?}", response);
        let response_body = response.bytes().await?;
        let response_body = serde_json::from_slice(&response_body)?;
        debug!("login response body: {:?}", response_body);
        Ok(response_body)
    }
}
