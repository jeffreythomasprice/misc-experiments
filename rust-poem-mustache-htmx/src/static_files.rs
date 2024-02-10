use include_dir::include_dir;

use poem::{endpoint, http::StatusCode, Endpoint, IntoResponse, Response};

use tracing::*;

use crate::http_utils::HttpError;

static STATIC_DIR: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

pub fn static_file(path: &'static str) -> impl Endpoint {
    fn f(path: &'static str) -> Result<Response, HttpError> {
        let file = STATIC_DIR.get_file(path).ok_or_else(|| {
            error!("no such static file: {path}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let contents = file.contents();
        let path = path.to_lowercase();
        let content_type = if path.ends_with(".html") || path.ends_with(".htm") {
            "text/html"
        } else if path.ends_with(".js") {
            "text/javascript"
        } else if path.ends_with(".css") {
            "text/css"
        } else {
            "text/plain"
        };
        Ok(contents.with_content_type(content_type).into_response())
    }

    endpoint::make(move |_| async move { f(path) })
}
