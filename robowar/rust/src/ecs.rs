use std::ops::AddAssign;

use tracing_subscriber::filter::targets::IntoIter;

struct Entity<T> {
    value: T,
    alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(usize);

pub struct ComponentVec<T> {
    entities: Vec<Entity<T>>,
}

pub struct ComponentVecIter<'a, T> {
    components: &'a ComponentVec<T>,
    next_index: Index,
}

impl AddAssign<usize> for Index {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl<T> ComponentVec<T> {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        match self.entities.get(index.0) {
            Some(Entity { value, alive }) => {
                if *alive {
                    Some(value)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.entities.get_mut(index.0) {
            Some(Entity { value, alive }) => {
                if *alive {
                    Some(value)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn push(&mut self, value: T) {
        self.entities.push(Entity { value, alive: true });
    }

    pub fn kill(&mut self, index: Index) {
        if let Some(Entity { value, alive }) = self.entities.get_mut(index.0) {
            *alive = false;
        }
    }
}

// TODO iterator methods on ComponentVec

impl<'a, T> Iterator for ComponentVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.components.get(self.next_index);
        self.next_index += 1;
        result
    }
}
