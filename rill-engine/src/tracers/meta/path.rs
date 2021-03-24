use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::path::{PathEvent, PathFlow, PathState};
use rill_protocol::io::provider::Path;

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct PathTracer {
    tracer: Tracer<PathFlow>,
}

impl PathTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let metric = PathFlow;
        let state = PathState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Add an path
    pub fn add(&self, name: Path) {
        let data = PathEvent::AddPath { name };
        self.tracer.send(data, None);
    }

    /// Remove an path
    pub fn del(&self, name: Path) {
        let data = PathEvent::RemovePath { name };
        self.tracer.send(data, None);
    }
}
