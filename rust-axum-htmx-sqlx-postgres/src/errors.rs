use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone)]
pub struct HttpError {
    status: StatusCode,
    message: String,
}

impl From<anyhow::Error> for HttpError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("{value:}"),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}
