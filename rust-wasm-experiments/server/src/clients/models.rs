use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}
