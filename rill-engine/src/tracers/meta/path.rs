use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::{
    path::{PathEvent, PathFlow, PathState},
    MetaFlow,
};
use rill_protocol::io::provider::{Description, Path};

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct PathTracer {
    tracer: Tracer<PathFlow>,
}

impl PathTracer {
    /// Create a new instance of the `Tracer`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = PathFlow::location();
        let metric = PathFlow;
        let state = PathState::new();
        let tracer = Tracer::new(metric, state, path, None);
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
