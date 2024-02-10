use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Cors, Tracing},
    web::{Data, Json},
    EndpointExt, Route, Server,
};
use shared::ClicksResponse;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ClicksService {
    count: Arc<Mutex<u64>>,
}

impl ClicksService {
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter("server=trace,poem=debug,debug")
        .init();

    let app = Route::new()
        .at("/click", get(get_clicks).post(click))
        .with(Cors::new())
        .with(Tracing)
        .with(AddData::new(ClicksService::new()));

    Server::new(TcpListener::bind("127.0.0.1:8001"))
        .run(app)
        .await
}

#[handler]
async fn get_clicks(clicks: Data<&ClicksService>) -> Json<ClicksResponse> {
    let count = clicks.count.lock().unwrap();
    Json(ClicksResponse { clicks: *count })
}

#[handler]
async fn click(clicks: Data<&ClicksService>) -> Json<ClicksResponse> {
    let mut count = clicks.count.lock().unwrap();
    *count += 1;
    Json(ClicksResponse { clicks: *count })
}
