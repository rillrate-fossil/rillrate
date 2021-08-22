use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Switch {
    tracer: BindedTracer<SwitchState>,
}

impl Switch {
    pub fn new(auto_path: impl Into<AutoPath>, spec: SwitchSpec) -> Self {
        let tracer = BindedTracer::new(auto_path, FlowMode::Realtime, spec);
        Self { tracer }
    }

    pub fn apply(&self, turn_on: bool) {
        let msg = SwitchEvent { turn_on };
        self.tracer.send(msg, None);
    }
}
