use std::collections::VecDeque;

static DEFAULT_SIZE: usize = 20;

#[derive(Debug)]
pub struct Frame<T> {
    size: usize,
    frame: VecDeque<T>,
}

impl<T> Default for Frame<T> {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE)
    }
}

impl<T> Frame<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            frame: VecDeque::with_capacity(size),
        }
    }

    pub fn insert(&mut self, item: T) -> Option<&T> {
        if self.frame.len() > self.size {
            self.frame.pop_front();
        }
        self.frame.push_back(item);
        self.frame.back()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.frame.iter()
    }
}
