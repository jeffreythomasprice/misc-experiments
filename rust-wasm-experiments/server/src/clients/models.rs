use std::fmt::Debug;

use shared::models::messages::ServerWebsocketMessage;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Clone)]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub sender: Option<Sender<ServerWebsocketMessage>>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("last_seen", &self.last_seen)
            // .field("sender", &self.sender)
            .finish()
    }
}
