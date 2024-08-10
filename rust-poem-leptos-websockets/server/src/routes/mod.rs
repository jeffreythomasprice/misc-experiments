use poem::{http::StatusCode, web::Json, IntoResponse};
use shared::ErrorResponse;

pub mod users;
pub mod websockets;

// TODO move me to a routes module
#[derive(Debug, Clone)]
struct StandardErrorResponse {
    pub status_code: StatusCode,
    pub body: ErrorResponse,
}

impl From<StatusCode> for StandardErrorResponse {
    fn from(value: StatusCode) -> Self {
        Self {
            status_code: value,
            body: ErrorResponse {
                message: match value.canonical_reason() {
                    Some(x) => x.to_owned(),
                    None => value.to_string(),
                },
            },
        }
    }
}

impl IntoResponse for StandardErrorResponse {
    fn into_response(self) -> poem::Response {
        (self.status_code, Json(self.body)).into_response()
    }
}

impl Into<poem::error::Error> for StandardErrorResponse {
    fn into(self) -> poem::error::Error {
        poem::error::Error::from_response(self.into_response())
    }
}
