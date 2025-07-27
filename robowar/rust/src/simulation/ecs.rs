use std::collections::HashMap;

use color_eyre::eyre::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub usize);

struct Component<T> {
    id: Id,
    data: T,
}

pub struct ComponentSystem<T> {
    data: HashMap<Id, Component<T>>,
    next_id: Id,
}

impl<T> ComponentSystem<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            next_id: Id(0),
        }
    }

    pub fn insert(&mut self, value: T) -> Id {
        let id = self.next_id;
        self.data.insert(id, Component { id, data: value });
        self.next_id = Id(self.next_id.0 + 1);
        id
    }

    pub fn insert_factory(&mut self, factory: impl FnOnce(Id) -> Result<T>) -> Result<Id> {
        let id = self.next_id;
        self.next_id = Id(self.next_id.0 + 1);
        self.data.insert(
            id,
            Component {
                id,
                data: factory(id)?,
            },
        );
        Ok(id)
    }

    pub fn get(&self, index: Id) -> Option<&T> {
        self.data.get(&index).map(|comp| &comp.data)
    }

    pub fn get_mut(&mut self, index: Id) -> Option<&mut T> {
        self.data.get_mut(&index).map(|comp| &mut comp.data)
    }

    pub fn remove(&mut self, index: Id) {
        self.data.remove(&index);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.data.iter().map(|(id, comp)| (*id, &comp.data))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> {
        self.data.iter_mut().map(|(id, comp)| (*id, &mut comp.data))
    }
}
