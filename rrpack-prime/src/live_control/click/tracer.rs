use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Click {
    #[deref]
    #[deref_mut]
    tracer: Tracer<ClickState>,
    _binder: Binder,
}

impl Click {
    pub fn new(auto_path: impl Into<AutoPath>, spec: ClickSpec) -> Self {
        let path = auto_path.into();
        let state = spec.into();
        let tracer = Tracer::new(state, path.into(), FlowMode::Realtime);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn apply(&self) {
        let msg = ClickEvent;
        self.tracer.send(msg, None);
    }
}
