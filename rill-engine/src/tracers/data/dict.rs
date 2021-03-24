use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::data::dict::{DictEvent, DictMetric, DictState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct DictTracer {
    tracer: Tracer<DictMetric>,
}

impl DictTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let metric = DictMetric;
        let state = DictState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Set a value to key.
    pub fn set(&self, key: String, value: String, timestamp: Option<SystemTime>) {
        let data = DictEvent::Assign { key, value };
        self.tracer.send(data, timestamp);
    }

    /// Remove a key.
    pub fn del(&self, key: String, timestamp: Option<SystemTime>) {
        let data = DictEvent::Remove { key };
        self.tracer.send(data, timestamp);
    }
}
