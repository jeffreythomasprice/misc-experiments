use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::models::{ClientHelloRequest, ClientHelloResponse};

#[derive(Clone)]
pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn client_hello(
        &self,
        request: &ClientHelloRequest,
    ) -> Result<ClientHelloResponse, reqwest::Error> {
        self.make_json_request_json_response(Method::POST, "client", request)
            .await
    }

    async fn make_json_request_json_response<RequestType, ResponseType>(
        &self,
        method: Method,
        path: &str,
        request: &RequestType,
    ) -> Result<ResponseType, reqwest::Error>
    where
        RequestType: Serialize,
        ResponseType: DeserializeOwned,
    {
        let client = reqwest::Client::new();
        let response = client
            .request(method, format!("{}/{}", self.base_url, path))
            .json(request)
            .send()
            .await?;
        let response_body: ResponseType = response.json().await?;
        Ok(response_body)
    }
}
