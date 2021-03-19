use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::counter::{CounterEvent, CounterMetric, CounterState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CounterTracer {
    tracer: Tracer<CounterMetric>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path) -> Self {
        let metric = CounterMetric;
        let state = CounterState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = CounterEvent::Inc(delta);
        self.tracer.send(data, timestamp);
    }
}
