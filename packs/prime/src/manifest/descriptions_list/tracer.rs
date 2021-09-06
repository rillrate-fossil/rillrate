use super::state::*;
use crate::manifest::description::PackFlowDescription;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;
use rill_protocol::io::provider::Path;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct PathsTracer {
    tracer: Tracer<PathsState>,
}

impl PathsTracer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = PathsSpec::path();
        let state = PathsSpec.into();
        let tracer = Tracer::new(state, path, FlowMode::Realtime);
        Self { tracer }
    }

    pub fn add_path(&self, path: Path, description: PackFlowDescription) {
        let msg = PathsEvent::Add { path, description };
        self.tracer.send(msg, None);
    }

    pub fn remove_path(&self, path: Path) {
        let msg = PathsEvent::Remove { path };
        self.tracer.send(msg, None);
    }
}
