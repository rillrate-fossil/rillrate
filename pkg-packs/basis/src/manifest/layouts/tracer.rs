use super::components::Layout;
use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;
use rill_protocol::io::provider::Path;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LayoutsTracer {
    tracer: Tracer<LayoutsState>,
}

impl LayoutsTracer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = LayoutsSpec::path();
        let state = LayoutsSpec.into();
        let tracer = Tracer::new(state, path, FlowMode::Realtime);
        Self { tracer }
    }

    pub fn add_tab(&self, name: Path, layout: Layout) {
        let msg = LayoutsEvent::Add { name, layout };
        self.tracer.send(msg, None);
    }

    pub fn remove_tab(&self, name: Path) {
        let msg = LayoutsEvent::Remove { name };
        self.tracer.send(msg, None);
    }
}
