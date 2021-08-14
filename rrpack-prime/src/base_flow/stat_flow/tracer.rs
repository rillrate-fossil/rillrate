use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct StatFlowTracer<T: StatFlowSpec> {
    tracer: Tracer<StatFlowState<T>>,
}

impl<T: StatFlowSpec> StatFlowTracer<T> {
    pub fn new(path: Path, spec: T) -> Self {
        // TODO: Consider to get `interval` from a parameter
        let interval = spec.interval();
        let state = StatFlowState::new(spec);
        let tracer = Tracer::new(state, path, interval, None);
        Self { tracer }
    }

    pub fn change(&self, delta: T::Delta) {
        let msg = StatFlowEvent::ApplyDelta { delta };
        self.tracer.send(msg, None);
    }
}
