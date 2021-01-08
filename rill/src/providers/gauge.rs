use super::ProtectedProvider;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Path, RillData, StreamType};
use std::time::SystemTime;

#[derive(Debug, Deref, DerefMut)]
pub struct GaugeProvider {
    #[deref]
    #[deref_mut]
    provider: ProtectedProvider<f64>,
}

impl GaugeProvider {
    pub fn new(path: Path) -> Self {
        let provider = ProtectedProvider::new(path, StreamType::GaugeStream, 0.0);
        Self { provider }
    }

    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.provider.lock() {
            *value += delta;
            if self.provider.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.provider.send(data, timestamp);
            }
        }
    }

    pub fn dec(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.provider.lock() {
            *value -= delta;
            if self.provider.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.provider.send(data, timestamp);
            }
        }
    }

    pub fn set(&self, new_value: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.provider.lock() {
            *value = new_value;
            if self.provider.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.provider.send(data, timestamp);
            }
        }
    }
}
