use std::net::SocketAddr;
use std::str::FromStr;

use axum::{routing::*, Json};
use shared::JsonResponse;
use tower::ServiceBuilder;
use tower_http::cors;

async fn root() -> String {
    return "Hello, World!".into();
}

async fn json_example() -> Json<JsonResponse> {
    Json(JsonResponse::new("baz", 42))
}

#[tokio::main]
async fn main() {
    let cors = cors::CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/json", get(json_example))
        .layer(ServiceBuilder::new().layer(cors));

    axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8001").unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
