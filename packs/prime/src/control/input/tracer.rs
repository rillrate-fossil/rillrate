use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Default)]
pub struct InputOpts {
    pub label: Option<String>,
}

impl From<InputOpts> for InputSpec {
    fn from(opts: InputOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_else(|| "Input".into()),
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