use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct MetaFlowTracer<T: MetaFlowSpec> {
    tracer: Tracer<MetaFlowState<T>>,
}

impl<T: MetaFlowSpec> MetaFlowTracer<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let state = MetaFlowState::new();
        let tracer = Tracer::new_push(state, T::path());
        Self { tracer }
    }

    pub fn set_meta(&self, meta: T::Meta) {
        let msg = MetaFlowEvent::SetMeta { meta };
        self.tracer.send(msg, None);
    }

    pub fn clear(&self) {
        let msg = MetaFlowEvent::Clear;
        self.tracer.send(msg, None);
    }
}
