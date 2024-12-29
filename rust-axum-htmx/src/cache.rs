use std::{collections::HashMap, future::Future, hash::Hash, sync::Arc};

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_create<F, E, Fut>(&self, key: K, f: F) -> Result<V, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        let data = &mut *self.data.lock().await;
        match data.entry(key) {
            std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                Ok(occupied_entry.get().clone())
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                let result = f().await?;
                vacant_entry.insert(result.clone());
                Ok(result)
            }
        }
    }
}
