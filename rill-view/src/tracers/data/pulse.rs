use crate::flow::data::pulse::{PulseEvent, PulseState};
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Sends metrics as `pulse` that can change value to any.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct PulseTracer {
    tracer: Tracer<PulseState>,
}

impl PulseTracer {
    /// Creates a new `Pulse` tracer.
    pub fn new(path: Path, depth: Option<u32>) -> Self {
        let state = PulseState::new(None, depth);
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Increments the value by the specific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = PulseEvent::Inc(delta);
        self.tracer.send(data, timestamp);
    }

    /// Decrements the value by the specific delta.
    pub fn dec(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = PulseEvent::Dec(delta);
        self.tracer.send(data, timestamp);
    }

    /// Set the value.
    pub fn set(&self, new_value: f64, timestamp: Option<SystemTime>) {
        let data = PulseEvent::Set(new_value);
        self.tracer.send(data, timestamp);
    }
}
