use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{timed, Tracer};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Pulse {
    #[deref]
    #[deref_mut]
    tracer: Tracer<PulseState>,
    _binder: Binder,
}

impl Pulse {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: PulseSpec) -> Self {
        let path = auto_path.into();
        let state = spec.into();
        let tracer = Tracer::new(state, path.into(), mode);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn push(&self, value: f64) {
        if let Some(value) = timed(value) {
            let msg = PulseEvent::Push { value };
            self.tracer.send(msg, None);
        }
    }
}
