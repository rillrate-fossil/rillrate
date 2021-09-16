use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Clone, Default)]
pub struct SwitchOpts {
    pub label: Option<String>,
}

impl From<SwitchOpts> for SwitchSpec {
    fn from(opts: SwitchOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_else(|| "Switch".into()),
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Switch {
    tracer: BindedTracer<SwitchState>,
}

impl Switch {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<SwitchSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self, turn_on: bool) {
        let msg = SwitchEvent { turn_on };
        self.tracer.send(msg, None);
    }
}
