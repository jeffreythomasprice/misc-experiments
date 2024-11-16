use axum::extract::FromRef;

use super::{kafka::Kafka, websockets::ConnectedClients};

#[derive(Clone)]
pub struct AppState {
    pub websockets: ConnectedClients,
    pub kafka: Kafka,
}

impl FromRef<AppState> for ConnectedClients {
    fn from_ref(input: &AppState) -> Self {
        input.websockets.clone()
    }
}

impl FromRef<AppState> for Kafka {
    fn from_ref(input: &AppState) -> Self {
        input.kafka.clone()
    }
}
