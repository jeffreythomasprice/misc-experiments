use super::APIService;

use anyhow::Result;
use leptos::SignalSet;
use shared::{CreateUserRequest, LogInRequest, LogInResponse, UserResponse};

impl APIService {
    pub async fn list_users(&self) -> Result<Vec<UserResponse>> {
        self.get_json_response("/users").await
    }

    pub async fn create_user(&self, request: &CreateUserRequest) -> Result<UserResponse> {
        self.post_json_request_json_response("/users", request)
            .await
    }

    pub async fn log_in(&self, request: &LogInRequest) -> Result<LogInResponse> {
        let result: LogInResponse = self
            .post_json_request_json_response("/login", request)
            .await?;
        self.auth_token.set(Some(result.token.clone()));
        log::debug!("logged in, token: {}", result.token);
        Ok(result)
    }
}
