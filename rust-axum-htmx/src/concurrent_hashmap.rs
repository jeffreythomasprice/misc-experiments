use std::{collections::HashMap, future::Future, hash::Hash, sync::Arc};

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ConcurrentHashMap<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> ConcurrentHashMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn values(&self) -> Vec<V> {
        let data = &mut *self.data.lock().await;
        data.values().map(|x| x.clone()).collect()
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let data = &mut *self.data.lock().await;
        data.get(key).cloned()
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let data = &mut *self.data.lock().await;
        data.insert(key, value)
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let data = &mut *self.data.lock().await;
        data.remove(key)
    }

    pub async fn get_or_insert<F, E, Fut>(&self, key: K, f: F) -> Result<V, E>
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
