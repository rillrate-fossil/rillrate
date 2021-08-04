use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{Tracer, Watcher};
use rill_protocol::io::provider::Path;

pub type StatFlowWatcher<T> = Watcher<StatFlowState<T>>;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct StatFlowTracer<T: StatFlowSpec> {
    tracer: Tracer<StatFlowState<T>>,
}

impl<T: StatFlowSpec> StatFlowTracer<T> {
    pub fn new(path: Path, spec: T) -> Self {
        let state = StatFlowState::new();
        let tracer = Tracer::new(state, path, spec.interval());
        Self { tracer }
    }

    pub fn change(&self, delta: T::Delta) {
        let msg = StatFlowEvent::ApplyDelta { delta };
        self.tracer.send(msg, None);
    }
}
