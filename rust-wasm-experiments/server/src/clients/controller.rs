use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use shared::models::messages::{CreateClientRequest, CreateClientResponse};

use crate::models::GenericErrorResponse;

use super::{Service, ServiceError};

impl From<ServiceError> for GenericErrorResponse {
    fn from(value: ServiceError) -> Self {
        // TODO different status codes for different kinds of errors
        GenericErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

pub async fn create(
    State(mut service): State<Service>,
    // TODO needs extract to get client id from auth header
    request: Json<CreateClientRequest>,
) -> Result<Json<CreateClientResponse>, GenericErrorResponse> {
    let result = service.create(request.name.clone())?;
    Ok(Json(CreateClientResponse {
        id: result.id.to_string(),
    }))
}

pub async fn websocket(
    State(mut service): State<Service>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // TODO needs a header to determine which client

    ws.on_upgrade(move |socket| handle_websocket(socket))
}

async fn handle_websocket(mut socket: WebSocket) {
    todo!()
}
