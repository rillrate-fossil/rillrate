use super::ProtectedProvider;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

/// Sends metrics as `gauge` that can change value to any.
#[derive(Debug, Deref, DerefMut)]
pub struct GaugeProvider {
    #[deref]
    #[deref_mut]
    provider: ProtectedProvider<f64>,
}

impl GaugeProvider {
    /// Creates a new `Gauge` provider.
    pub fn new(path: Path) -> Self {
        let info = format!("{} gauge", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::GaugeStream,
        };
        let provider = ProtectedProvider::new(description, 0.0);
        Self { provider }
    }

    /// Increments the value by the specific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.provider.lock() {
            *value += delta;
            if self.provider.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.provider.send(data, timestamp);
            }
        }
    }

    /// Decrements the value by the specific delta.
    pub fn dec(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.provider.lock() {
            *value -= delta;
            if self.provider.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.provider.send(data, timestamp);
            }
        }
    }

    /// Set the value.
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
