use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Selector {
    #[deref]
    #[deref_mut]
    tracer: Tracer<SelectorState>,
    _binder: Binder,
}

impl Selector {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        mode: FlowMode,
        label: impl ToString,
        options: Vec<String>,
    ) -> Self {
        let path = auto_path.into();
        let state = SelectorState::new(label.to_string(), options);
        let tracer = Tracer::new(state, path.into(), mode);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn apply(&self, value: Option<String>) {
        let msg = SelectorEvent {
            update_selected: value,
        };
        self.tracer.send(msg, None);
    }
}
