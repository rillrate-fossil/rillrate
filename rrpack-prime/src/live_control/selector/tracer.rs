use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Selector {
    tracer: BindedTracer<SelectorState>,
}

impl Selector {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<SelectorSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self, value: Option<String>) {
        let msg = SelectorEvent {
            update_selected: value,
        };
        self.tracer.send(msg, None);
    }
}
