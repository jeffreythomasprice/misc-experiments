use axum::{extract::State, http::StatusCode, Json};
use shared::models::messages::{ClientHelloRequest, ClientHelloResponse, GenericResponse};

use super::{Service, ServiceError};

type GenericErrorResponse = (StatusCode, Json<GenericResponse>);

impl From<ServiceError> for GenericErrorResponse {
    fn from(value: ServiceError) -> Self {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenericResponse {
                message: value.to_string(),
            }),
        )
    }
}

pub async fn create(
    State(mut service): State<Service>,
    request: Json<ClientHelloRequest>,
) -> Result<Json<ClientHelloResponse>, GenericErrorResponse> {
    let result = service.create(request.name.clone())?;
    Ok(Json(ClientHelloResponse {
        id: result.id.to_string(),
    }))
}
