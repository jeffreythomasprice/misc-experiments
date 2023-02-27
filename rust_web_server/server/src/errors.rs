use rocket::{http::Status, response::Responder, serde::json::Json, Catcher, Response};
use shared::errors::ErrorResponse;

const PLACEHOLDER_MESSAGE: &str = "uncaught, no details available";

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

    pub fn cache_on_request(&self, request: &rocket::Request) {
        trace!("caching error {:?}", self.clone());
        request.local_cache(move || Some(self.clone()));
    }

    pub fn get_cached_from_request(request: &rocket::Request, default: Error) -> Error {
        match request.local_cache(move || None::<Error>) {
            Some(result) => result.clone(),
            None => {
                warn!(
                    "expected cached error but there was none, using provided default {:?}",
                    default
                );
                default
            }
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::InternalServerError(value.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::InternalServerError(value.to_string())
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        let (status, response) = self.to_response();
        trace!("responding {status} {response:?}");
        Response::build_from(Json(response).respond_to(request)?)
            .status(status)
            .ok()
    }
}

pub fn catchers() -> Vec<Catcher> {
    catchers![unauthorized, forbidden, not_found, internal_server_error]
}

#[catch(401)]
fn unauthorized(request: &rocket::Request) -> Error {
    Error::get_cached_from_request(request, Error::Unauthorized)
}

#[catch(403)]
fn forbidden(request: &rocket::Request) -> Error {
    Error::get_cached_from_request(request, Error::Forbidden)
}

#[catch(404)]
fn not_found(request: &rocket::Request) -> Error {
    Error::get_cached_from_request(request, Error::NotFound(PLACEHOLDER_MESSAGE.to_string()))
}

#[catch(500)]
fn internal_server_error(request: &rocket::Request) -> Error {
    Error::get_cached_from_request(
        request,
        Error::InternalServerError(PLACEHOLDER_MESSAGE.to_string()),
    )
}
