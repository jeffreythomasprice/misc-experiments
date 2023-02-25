use rocket::{http::Status, response::status, serde::json::Json};
use shared::errors::ErrorResponse;

#[derive(Debug)]
pub enum Error {
    Sql(sqlx::Error),
    NotFound(String),
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

pub type JsonResult<T> = Result<Json<T>, status::Custom<Json<ErrorResponse>>>;

pub fn get_json_result<T, S>(r: Result<S, Error>) -> JsonResult<T>
where
    S: Into<T>,
{
    match r {
        Ok(result) => Ok(Json(result.into())),
        Err(e) => match e {
            Error::Sql(_) => Err(status::Custom(
                Status::InternalServerError,
                Json(ErrorResponse::new(&format!("{e:?}"))),
            )),
            Error::NotFound(message) => Err(status::Custom(
                Status::NotFound,
                Json(ErrorResponse::new(&message)),
            )),
        },
    }
}
