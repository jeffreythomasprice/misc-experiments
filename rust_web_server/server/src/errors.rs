use rocket::{http::Status, response::Responder, serde::json::Json, Response};
use shared::errors::ErrorResponse;

#[derive(Debug)]
pub enum Error {
    Sql(sqlx::Error),
    NotFound(String),
    Unauthorized,
    Forbidden,
}

impl Error {
    pub fn to_response(&self) -> (Status, ErrorResponse) {
        match self {
            Error::Sql(e) => (
                Status::InternalServerError,
                ErrorResponse::new(&format!("{e:?}")),
            ),
            Error::NotFound(message) => (Status::NotFound, ErrorResponse::new(&message)),
            Error::Unauthorized => (Status::Unauthorized, ErrorResponse::new("unauthorized")),
            Error::Forbidden => (Status::Forbidden, ErrorResponse::new("forbidden")),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // TODO smarter comparison of sqlx errors
            (Self::Sql(l0), Self::Sql(r0)) => l0.to_string() == r0.to_string(),
            (Self::NotFound(l0), Self::NotFound(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Sql(value)
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
