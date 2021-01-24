use super::ProtectedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut)]
pub struct CounterTracer {
    #[deref]
    #[deref_mut]
    tracer: ProtectedTracer<f64>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path) -> Self {
        let info = format!("{} counter", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::CounterStream,
        };
        let tracer = ProtectedTracer::new(description, 0.0);
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.tracer.lock() {
            *value += delta;
            if self.tracer.is_active() {
                let data = RillData::CounterRecord { value: *value };
                self.tracer.send(data, timestamp);
            }
        }
    }
}
