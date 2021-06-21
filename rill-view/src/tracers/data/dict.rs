use crate::flow::data::dict::{DictEvent, DictState};
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct DictTracer {
    tracer: Tracer<DictState>,
}

impl DictTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let state = DictState::new();
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Set a value to key.
    pub fn set(&self, key: String, value: String, timestamp: Option<SystemTime>) {
        let data = DictEvent::Assign { key, value };
        self.tracer.send(data, timestamp, None);
    }

    /// Remove a key.
    pub fn del(&self, key: String, timestamp: Option<SystemTime>) {
        let data = DictEvent::Remove { key };
        self.tracer.send(data, timestamp, None);
    }
}
