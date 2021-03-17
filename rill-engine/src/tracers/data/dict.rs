use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::dict::{DictEvent, DictMetric, DictState};
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
        let state = DictState::new();
        let tracer = Tracer::new(state, path, None);
        Self { tracer }
    }

    /// Set a value to key.
    pub fn set(&self, key: impl ToString, value: impl ToString, timestamp: Option<SystemTime>) {
        let data = DictEvent::Assign {
            key: key.to_string(),
            value: value.to_string(),
        };
        self.tracer.send(data, timestamp);
    }

    /// Remove a key.
    pub fn del(&self, key: impl ToString, timestamp: Option<SystemTime>) {
        let data = DictEvent::Remove {
            key: key.to_string(),
        };
        self.tracer.send(data, timestamp);
    }
}
