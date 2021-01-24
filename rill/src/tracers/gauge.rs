use super::ProtectedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

/// Sends metrics as `gauge` that can change value to any.
#[derive(Debug, Deref, DerefMut)]
pub struct GaugeTracer {
    #[deref]
    #[deref_mut]
    tracer: ProtectedTracer<f64>,
}

impl GaugeTracer {
    /// Creates a new `Gauge` tracer.
    pub fn new(path: Path) -> Self {
        let info = format!("{} gauge", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::GaugeStream,
        };
        let tracer = ProtectedTracer::new(description, 0.0);
        Self { tracer }
    }

    /// Increments the value by the specific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.tracer.lock() {
            *value += delta;
            if self.tracer.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.tracer.send(data, timestamp);
            }
        }
    }

    /// Decrements the value by the specific delta.
    pub fn dec(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.tracer.lock() {
            *value -= delta;
            if self.tracer.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.tracer.send(data, timestamp);
            }
        }
    }

    /// Set the value.
    pub fn set(&self, new_value: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.tracer.lock() {
            *value = new_value;
            if self.tracer.is_active() {
                let data = RillData::GaugeValue { value: *value };
                self.tracer.send(data, timestamp);
            }
        }
    }
}
