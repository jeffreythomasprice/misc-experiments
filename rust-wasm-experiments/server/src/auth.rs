use std::str::FromStr;

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
use uuid::Uuid;

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
            // TODO should use random bytes
            key: Hmac::<Sha256>::new_from_slice(b"foobar")?,
        })
    }

    pub fn create(&self, client: &Client) -> Result<String, SigningError> {
        let claims = Claims {
            id: client.id.to_string(),
        };
        let signed_jwt_string = claims
            .sign_with_key(&self.key)
            .map_err(|e| SigningError(format!("failed to sign jwt: {e:?}")))?;
        Ok(signed_jwt_string)
    }

    pub fn validate(&self, token: &str) -> Result<Claims, ValidationError> {
        token
            .verify_with_key(&self.key)
            .map_err(|e| ValidationError(format!("failed to validate token: {e:?}")))
    }
}

pub async fn middleware<T>(
    State(auth_service): State<Service>,
    State(clients_service): State<clients::Service>,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    request: Request<T>,
    next: Next<T>,
) -> Result<Response, GenericErrorResponse> {
    match auth {
        Some(auth) => {
            let claims = auth_service.validate(auth.token())?;
            trace!("got claims from auth token: {claims:?}");

            let id = match Uuid::from_str(&claims.id) {
                Ok(id) => id,
                Err(e) => {
                    error!("error parsing client claims, while parsing id into a uiud: {e:?}");
                    // TODO deduplicate these
                    Err(GenericErrorResponse::new(
                        StatusCode::UNAUTHORIZED,
                        "unauthorized".into(),
                    ))?
                }
            };
            debug!("parsed id as uuid: {id}");

            let client = clients_service.get_by_id(id).await;
            trace!("got client from claims: {client:?}");

            match client {
                Some(_) => {
                    let response = next.run(request).await;
                    Ok(response)
                }
                // TODO deduplicate these
                None => Err(GenericErrorResponse::new(
                    StatusCode::UNAUTHORIZED,
                    "unauthorized".into(),
                ))?,
            }
        }
        // TODO deduplicate these
        None => Err(GenericErrorResponse::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized".into(),
        )),
    }
}
