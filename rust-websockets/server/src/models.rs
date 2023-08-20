use axum::{http::StatusCode, response::IntoResponse, Json};
use shared::models::messages::GenericResponse;

pub struct GenericErrorResponse((StatusCode, Json<GenericResponse>));

impl GenericErrorResponse {
    pub fn new(status_code: StatusCode, message: String) -> Self {
        GenericErrorResponse((status_code, Json(GenericResponse { message })))
    }
}

impl IntoResponse for GenericErrorResponse {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}
