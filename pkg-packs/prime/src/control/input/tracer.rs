use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct InputOpts {
    pub label: Option<String>,
    pub wide: Option<bool>,
    pub password: Option<bool>,
    pub placeholder: Option<String>,
}

impl From<InputOpts> for InputSpec {
    fn from(opts: InputOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_default(),
            wide: opts.wide.unwrap_or_default(),
            password: opts.password.unwrap_or_default(),
            placeholder: opts.placeholder.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Input {
    tracer: BindedTracer<InputState>,
}

impl Input {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<InputSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self, value: impl Into<String>) {
        let msg = InputEvent {
            changed_text: value.into(),
        };
        self.tracer.send(msg, None);
    }
}
