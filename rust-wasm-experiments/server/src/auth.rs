use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    TypedHeader,
};
use hmac::{
    digest::{InvalidLength, KeyInit},
    Hmac,
};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::*;

use crate::{
    clients::{self, Client},
    models::GenericErrorResponse,
};

#[derive(Clone)]
pub struct Service {
    key: Hmac<Sha256>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
}

#[derive(Debug)]
pub struct SigningError(String);

impl From<SigningError> for GenericErrorResponse {
    fn from(value: SigningError) -> Self {
        GenericErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, value.0)
    }
}

#[derive(Debug)]
pub struct ValidationError(String);

impl From<ValidationError> for GenericErrorResponse {
    fn from(_value: ValidationError) -> Self {
        GenericErrorResponse::new(StatusCode::UNAUTHORIZED, "unauthorized".into())
    }
}

impl Service {
    pub fn new() -> Result<Self, InvalidLength> {
        Ok(Self {
            key: Hmac::<Sha256>::new_from_slice(b"foobar")?,
        })
    }

    pub fn create(&self, client: &Client) -> Result<String, SigningError> {
        let claims = Claims {
            id: client.id.to_string(),
        };
        let signed_jwt_string = claims
            .sign_with_key(&self.key)
            .or_else(|e| Err(SigningError(format!("failed to sign jwt: {e:?}"))))?;
        Ok(signed_jwt_string)
    }

    pub fn validate(&self, token: &str) -> Result<Claims, ValidationError> {
        token
            .verify_with_key(&self.key)
            .or_else(|e| Err(ValidationError(format!("failed to validate token: {e:?}"))))
    }
}

pub async fn middleware<T>(
    State(auth_service): State<Service>,
    State(clients_service): State<clients::Service>,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    request: Request<T>,
    next: Next<T>,
) -> Result<Response, GenericErrorResponse> {
    info!("TODO JEFF should check auth header {:?}", auth);

    match auth {
        Some(auth) => {
            let claims = auth_service.validate(auth.token())?;
            info!("TODO JEFF got claims from auth token: {claims:?}");
            // TODO look up client from claims

            let response = next.run(request).await;
            Ok(response)
        }
        None => Err(GenericErrorResponse::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized".into(),
        )),
    }
}

// TODO an extractor to get client id from auth header
