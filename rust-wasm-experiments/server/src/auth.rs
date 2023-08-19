use axum::{
    headers::{authorization::Bearer, Authorization},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json, TypedHeader,
};
use tracing::*;

use crate::models::GenericErrorResponse;

pub async fn jwt_auth<T>(
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    request: Request<T>,
    next: Next<T>,
) -> Result<Response, GenericErrorResponse> {
    info!("TODO JEFF should check auth header {:?}", auth);

    match auth {
        Some(auth) => {
            // TODO actually check header

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
