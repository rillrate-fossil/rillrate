use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::path::{PathEvent, PathState};
use rill_protocol::io::provider::{Description, Path};

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct PathTracer {
    tracer: Tracer<PathState>,
}

impl PathTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path, description: Description) -> Self {
        let state = PathState::new(description);
        // TODO: Use the receiver
        let tracer = Tracer::new_push(state, path).0;
        Self { tracer }
    }

    /// Add an path
    pub fn add(&self, path: Path, description: Description) {
        let data = PathEvent::AddPath { path, description };
        self.tracer.send(data, None);
    }

    /// Remove an path
    pub fn del(&self, path: Path) {
        let data = PathEvent::RemovePath { path };
        self.tracer.send(data, None);
    }
}
