use super::ProtectedProvider;
use crate::protocol::{Path, RillData, StreamType};
use derive_more::{Deref, DerefMut};
use std::time::SystemTime;

#[derive(Debug, Deref, DerefMut)]
pub struct CounterProvider {
    #[deref]
    #[deref_mut]
    provider: ProtectedProvider<f64>,
}

impl CounterProvider {
    pub fn new(path: Path) -> Self {
        let provider = ProtectedProvider::new(path, StreamType::CounterStream, 0.0);
        Self { provider }
    }

    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.provider.lock() {
            *value += delta;
            if self.provider.is_active() {
                let data = RillData::CounterRecord { value: *value };
                self.provider.send(data, timestamp);
            }
        }
    }
}
