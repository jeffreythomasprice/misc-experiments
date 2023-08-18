use std::fmt::Debug;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, net::SocketAddr};

use axum::extract::State;
use axum::{routing::*, Json};
use shared::models::messages::{ClientHelloRequest, GenericResponse};
use tower::ServiceBuilder;
use tower_http::{cors, trace::TraceLayer};
use tracing::*;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

struct Client {
    id: Uuid,
    name: String,
    last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct AppState {
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

impl AppState {
    #[instrument]
    pub fn cleanup(&self) {
        trace!("running cleanup");

        // TODO error handling
        let mut clients = self.clients.lock().unwrap();

        let now = chrono::Utc::now();
        let expiry_time = now - chrono::Duration::seconds(10);
        clients.retain(|id, client| {
            if client.last_seen < expiry_time {
                debug!(
                    "expiring {}, last seen {:?}",
                    id,
                    client.last_seen.to_rfc3339()
                );
                false
            } else {
                true
            }
        });
    }
}

async fn client_hello(
    State(state): State<AppState>,
    request: Json<ClientHelloRequest>,
) -> Json<GenericResponse> {
    info!("request = {request:?}");

    let id = Uuid::new_v4();
    debug!("assigned id {id}");

    let client = Client {
        id,
        name: request.name.clone(),
        last_seen: chrono::Utc::now(),
    };
    // TODO error handling
    let mut clients = state.clients.lock().unwrap();
    if clients.contains_key(&id) {
        // TODO return error
        error!("already contains key {id}");
    }
    clients.insert(id, client);
    trace!("there are now {} clients", clients.len());

    Json(GenericResponse::ok())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_str(
                vec![
                    // defaults first
                    "info".to_string(),
                    "tower_http=debug".to_string(),
                    "server=debug".to_string(),
                    // respect the env var to override
                    match std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV) {
                        Ok(env_value) => env_value,
                        Err(_) => "".to_string(),
                    },
                ]
                .join(",")
                .as_str(),
            )
            .unwrap(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = cors::CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let state = AppState {
        clients: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/client", post(client_hello))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;

            state.cleanup();
        }
    });

    axum::Server::bind(&SocketAddr::from_str("127.0.0.1:8001").unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
