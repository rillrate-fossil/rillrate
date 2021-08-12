use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{Tracer, Watcher};
use rill_protocol::io::provider::Path;

pub type EmitControlWatcher<T> = Watcher<EmitControlState<T>>;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct EmitControlTracer<T: EmitControlSpec> {
    tracer: Tracer<EmitControlState<T>>,
}

impl<T: EmitControlSpec> EmitControlTracer<T> {
    pub fn new(path: Path, spec: T, initial_state: T::State) -> Self {
        let state = EmitControlState::new(spec, initial_state);
        // TODO: Spawn a meio actor for watcher?
        let (tracer, watcher) = Tracer::new_push(state, path);
        Self { tracer }
    }
}
