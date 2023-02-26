use rocket::{http::Status, response::Responder, serde::json::Json, Response};
use shared::errors::ErrorResponse;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InternalServerError(String),
    NotFound(String),
    Unauthorized,
    Forbidden,
}

impl Error {
    pub fn to_response(&self) -> (Status, ErrorResponse) {
        match self {
            Error::InternalServerError(message) => {
                (Status::InternalServerError, ErrorResponse::new(&message))
            }
            Error::NotFound(message) => (Status::NotFound, ErrorResponse::new(&message)),
            Error::Unauthorized => (Status::Unauthorized, ErrorResponse::new("unauthorized")),
            Error::Forbidden => (Status::Forbidden, ErrorResponse::new("forbidden")),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::InternalServerError(value.to_string())
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        let (status, response) = self.to_response();
        Response::build_from(Json(response).respond_to(request)?)
            .status(status)
            .ok()
    }
}
