use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

#[derive(Debug)]
pub enum CounterDelta {
    Increment(f64),
}

impl TracerEvent for CounterDelta {}

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut)]
pub struct CounterTracer {
    #[deref]
    #[deref_mut]
    tracer: Tracer<CounterDelta>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path, active: bool) -> Self {
        let info = format!("{} counter", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::CounterStream,
        };
        let tracer = Tracer::new(description, active);
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = CounterDelta::Increment(delta);
        self.tracer.send(data, timestamp);
        /* TODO: Remove
        if let Some(mut value) = self.tracer.lock() {
            *value += delta;
            if self.tracer.is_active() {
                let data = CounterDelta::Increment(delta);
                self.tracer.send(data, timestamp);
            }
        }
        */
    }
}
