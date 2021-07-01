use derive_more::Deref;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

static DEFAULT_SIZE: u32 = 20;

#[derive(Debug, Clone, Serialize, Deserialize, Deref)]
pub struct Frame<T> {
    /// Size specified to have predictable seialization behavior.
    size: u32,
    #[deref]
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

    /*
    /// Returns a reference to the inserted element.
    pub fn insert(&mut self, item: T) -> &T {
        self.insert_pop(item);
        self.frame.back().unwrap()
    }
    */

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

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn clear(&mut self) {
        self.frame.clear()
    }
}
