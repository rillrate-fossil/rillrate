use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct EmitControlTracer<T: EmitControlSpec> {
    tracer: Tracer<EmitControlState<T>>,
}

impl<T: EmitControlSpec> EmitControlTracer<T> {
    pub fn new(path: Path, spec: T, initial_state: T::State) -> Self {
        let state = EmitControlState::new(spec, initial_state);
        let tracer = Tracer::new_push(state, path, None);
        Self { tracer }
    }
}
