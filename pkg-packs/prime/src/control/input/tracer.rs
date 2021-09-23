use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct InputOpts {
    pub label: Option<String>,
    pub wide: Option<bool>,
}

impl From<InputOpts> for InputSpec {
    fn from(opts: InputOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_else(|| "Input".into()),
            wide: opts.wide.unwrap_or_default(),
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
}
