use poem::{http::StatusCode, web::headers::ContentType, IntoResponse, Response};

pub type HttpError = StatusCode;

pub fn to_html_response(s: impl Into<String>) -> Response {
    s.into()
        .with_content_type(ContentType::html().to_string())
        .into_response()
}
