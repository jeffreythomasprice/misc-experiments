use std::{
    collections::{LinkedList, VecDeque},
    sync::{Arc, Mutex},
};

use color_eyre::eyre::Result;

pub struct Pool<T> {
    not_in_use: Arc<Mutex<VecDeque<T>>>,
    allocator: Arc<dyn Fn() -> Result<T>>,
}

impl<T> Pool<T>
where
    T: 'static,
{
    pub fn new(allocator: impl Fn() -> Result<T> + 'static) -> Self {
        Self {
            not_in_use: Arc::new(Mutex::new(VecDeque::new())),
            allocator: Arc::new(allocator),
        }
    }

    pub fn arena(&mut self) -> Arena<T> {
        let allocator = self.allocator.clone();
        Arena::new(self.not_in_use.clone(), move || allocator())
    }
}

pub struct Arena<T> {
    not_in_use: Arc<Mutex<VecDeque<T>>>,
    allocator: Box<dyn Fn() -> Result<T>>,
    in_use: LinkedList<T>,
}

impl<T> Arena<T> {
    fn new(
        not_in_use: Arc<Mutex<VecDeque<T>>>,
        allocator: impl Fn() -> Result<T> + 'static,
    ) -> Self {
        Self {
            not_in_use,
            allocator: Box::new(allocator),
            in_use: LinkedList::new(),
        }
    }

    pub fn get(&mut self) -> Result<&T> {
        let mut not_in_use = self.not_in_use.lock().unwrap();

        let result = if let Some(result) = not_in_use.pop_back() {
            result
        } else {
            (self.allocator)()?
        };

        self.in_use.push_back(result);
        Ok(self.in_use.back().unwrap())
    }

    pub fn get_mut(&mut self) -> Result<&mut T> {
        let mut not_in_use = self.not_in_use.lock().unwrap();

        let result = if let Some(result) = not_in_use.pop_back() {
            result
        } else {
            (self.allocator)()?
        };

        self.in_use.push_back(result);
        Ok(self.in_use.back_mut().unwrap())
    }
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        let mut not_in_use = self.not_in_use.lock().unwrap();
        while let Some(x) = self.in_use.pop_back() {
            not_in_use.push_back(x);
        }
    }
}
