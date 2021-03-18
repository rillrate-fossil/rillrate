use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

static DEFAULT_SIZE: u32 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame<T> {
    /// Size specified to have predictable seialization behavior.
    size: u32,
    frame: VecDeque<T>,
}

impl<T> Default for Frame<T> {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE)
    }
}

impl<T> Frame<T> {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            frame: VecDeque::with_capacity(size as usize),
        }
    }

    /// Returns a reference to the inserted element.
    pub fn insert(&mut self, item: T) -> &T {
        self.insert_pop(item);
        self.frame.back().unwrap()
    }

    pub fn insert_pop(&mut self, item: T) -> Option<T> {
        let result = {
            if self.frame.len() > self.size as usize {
                self.frame.pop_front()
            } else {
                None
            }
        };
        self.frame.push_back(item);
        result
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.frame.iter()
    }

    pub fn last(&self) -> Option<&T> {
        if !self.frame.is_empty() {
            self.frame.get(self.frame.len() - 1)
        } else {
            None
        }
    }
}
