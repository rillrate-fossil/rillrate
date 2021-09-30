use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct SwitchOpts {
    pub label: Option<String>,
}

impl From<SwitchOpts> for SwitchSpec {
    fn from(opts: SwitchOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_default(),
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
