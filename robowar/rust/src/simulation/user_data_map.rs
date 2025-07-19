use std::collections::HashMap;

use color_eyre::eyre::Result;

pub struct UserDataMap<T> {
    data: HashMap<u128, T>,
    next: u128,
}

impl<T> UserDataMap<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            // user data in rapier is a u128, and defaults to 0, so we start at 1
            next: 1,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u128, &T)> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u128, &mut T)> {
        self.data.iter_mut()
    }

    pub fn get(&self, id: u128) -> Option<&T> {
        self.data.get(&id)
    }

    pub fn get_mut(&mut self, id: u128) -> Option<&mut T> {
        self.data.get_mut(&id)
    }

    pub fn insert(&mut self, value: T) -> u128 {
        let id = self.next;
        self.data.insert(id, value);
        self.next += 1;
        id
    }

    pub fn insert_factory(&mut self, factory: impl FnOnce(u128) -> Result<T>) -> Result<u128> {
        let id = self.next;
        self.next += 1;
        self.data.insert(id, factory(id)?);
        Ok(id)
    }

    pub fn remove(&mut self, id: u128) -> Option<T> {
        self.data.remove(&id)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
