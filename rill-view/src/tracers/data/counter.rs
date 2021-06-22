use crate::flow::data::counter::{CounterEvent, CounterState};
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CounterTracer {
    tracer: Tracer<CounterState>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path) -> Self {
        let state = CounterState::new();
        let tracer = Tracer::new_push(state, path).0;
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = CounterEvent::Inc(delta);
        self.tracer.send(data, timestamp, None);
    }
}
