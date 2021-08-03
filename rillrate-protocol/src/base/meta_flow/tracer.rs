use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{Tracer, Watcher};

pub type MetaFlowWatcher<T> = Watcher<MetaFlowState<T>>;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct MetaFlowTracer<T: MetaFlowSpec> {
    tracer: Tracer<MetaFlowState<T>>,
}

impl<T: MetaFlowSpec> MetaFlowTracer<T> {
    pub fn new() -> (Self, MetaFlowWatcher<T>) {
        let state = MetaFlowState::new();
        let (tracer, watcher) = Tracer::new_push(state, T::path());
        (Self { tracer }, watcher)
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
