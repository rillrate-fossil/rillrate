pub mod pathfinder;
pub mod provider;

use std::sync::atomic::{AtomicU16, Ordering};

metacrate::meta!();

pub static PORT: Port = Port::new(1636);

pub struct Port {
    value: AtomicU16,
}

impl Port {
    const fn new(value: u16) -> Self {
        Self {
            value: AtomicU16::new(value),
        }
    }

    pub fn set(&self, value: u16) {
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> u16 {
        self.value.load(Ordering::Relaxed)
    }
}
