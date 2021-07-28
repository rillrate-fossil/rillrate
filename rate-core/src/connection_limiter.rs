use meio::{Actor, Address, IdOf};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Limit {
    pub total: usize,
}

impl Limit {
    pub const fn new(total: usize) -> Self {
        Self { total }
    }

    pub const fn max() -> Self {
        Self::new(usize::MAX)
    }
}

#[derive(Debug)]
pub struct ConnectionLimiter<T: Actor> {
    slots: HashMap<IdOf<T>, Address<T>>,
    limit: Limit,
}

impl<T: Actor> ConnectionLimiter<T> {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
            limit: Limit::default(),
        }
    }

    pub fn has_slot(&self) -> bool {
        self.slots.len() < self.limit.total
    }

    pub fn limit(&self) -> &Limit {
        &self.limit
    }

    /// Returns list of actors that have to be interrupted.
    pub fn set_limit(&mut self, new_limit: Limit) -> Vec<Address<T>> {
        self.limit = new_limit;
        let len = self.slots.len();
        if len > self.limit.total {
            let diff = len - self.limit.total;
            self.slots.values().take(diff).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn acquire(&mut self, address: Address<T>) {
        self.slots.insert(address.id(), address);
    }

    pub fn release(&mut self, item: IdOf<T>) {
        self.slots.remove(&item);
    }
}
