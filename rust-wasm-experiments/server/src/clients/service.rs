use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use shared::models::messages::ServerWebsocketMessage;
use tokio::sync::mpsc::Sender;
use tracing::*;
use uuid::Uuid;

use super::Client;

#[derive(Debug)]
pub enum ServiceError {
    DuplicateId(Uuid),
    NoSuchId(Uuid),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ServiceError::DuplicateId(id) => format!("duplicate id {id}"),
                ServiceError::NoSuchId(id) => format!("no such id {id}"),
            }
        )
    }
}

#[derive(Clone)]
pub struct Service {
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
}

impl Service {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[instrument]
    pub fn get_by_id(&self, id: Uuid) -> Option<Client> {
        let clients = self.clients.lock().unwrap();
        clients.get(&id).cloned()
    }

    #[instrument(ret)]
    pub fn create(&mut self, name: String) -> Result<Client, ServiceError> {
        info!("creating new client");

        let mut clients = self.clients.lock().unwrap();

        let id = Uuid::new_v4();
        debug!("assigned id {id}");

        let result = Client {
            id,
            name,
            last_seen: chrono::Utc::now(),
            sender: None,
        };

        if clients.contains_key(&id) {
            error!("already contains key {id}");
            return Err(ServiceError::DuplicateId(id));
        }
        clients.insert(id, result.clone());

        Ok(result)
    }

    #[instrument(ret, skip(sender))]
    pub fn update_with_sender(
        &mut self,
        id: Uuid,
        sender: Sender<ServerWebsocketMessage>,
    ) -> Result<(), ServiceError> {
        info!("updating client with sender");

        let mut clients = self.clients.lock().unwrap();

        let client = clients.get_mut(&id).ok_or(ServiceError::NoSuchId(id))?;
        client.sender = Some(sender);

        Ok(())
    }

    #[instrument]
    pub fn update_last_seen_time(&mut self, id: Uuid) -> Result<(), ServiceError> {
        let mut clients = self.clients.lock().unwrap();

        let client = clients.get_mut(&id).ok_or(ServiceError::NoSuchId(id))?;
        client.last_seen = chrono::Utc::now();

        Ok(())
    }

    #[instrument(ret)]
    pub fn delete(&mut self, id: Uuid) -> Option<Client> {
        info!("deleting");

        let mut clients = self.clients.lock().unwrap();

        clients.remove(&id)
    }

    #[instrument]
    pub fn cleanup(&mut self) {
        trace!("running cleanup");

        let mut clients = self.clients.lock().unwrap();

        let now = chrono::Utc::now();
        let expiry_time = now - chrono::Duration::seconds(10);
        clients.retain(|id, client| {
            if client.last_seen < expiry_time {
                debug!(
                    "expiring id={}, name={}, last seen {:?}",
                    id,
                    client.name,
                    client.last_seen.to_rfc3339()
                );
                false
            } else {
                true
            }
        });
        trace!("there are now {} clients", clients.len());
    }
}

impl Debug for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Service").finish()
    }
}
