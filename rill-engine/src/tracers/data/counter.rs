use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::pulse::{PulseEvent, PulseMetric, PulseState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CounterTracer {
    tracer: Tracer<PulseMetric>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path) -> Self {
        let metric = PulseMetric;
        let state = PulseState::new(None, None);
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = PulseEvent::Increment(delta);
        self.tracer.send(data, timestamp);
    }
}
