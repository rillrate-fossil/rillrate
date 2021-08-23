use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;
use rill_protocol::io::provider::{Description, Path};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct DescriptionsListTracer {
    tracer: Tracer<DescriptionsListState>,
}

impl DescriptionsListTracer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = DescriptionsListSpec::path();
        let state = DescriptionsListSpec.into();
        let tracer = Tracer::new(state, path, FlowMode::Realtime);
        Self { tracer }
    }

    pub fn add_path(&self, path: Path, description: Description) {
        let msg = DescriptionsListEvent::Add { path, description };
        self.tracer.send(msg, None);
    }

    pub fn remove_path(&self, path: Path) {
        let msg = DescriptionsListEvent::Remove { path };
        self.tracer.send(msg, None);
    }
}
