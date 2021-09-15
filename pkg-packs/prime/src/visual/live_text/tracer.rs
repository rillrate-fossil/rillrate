use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Default)]
pub struct LiveTextOpts {}

impl From<LiveTextOpts> for LiveTextSpec {
    fn from(_opts: LiveTextOpts) -> Self {
        Self {}
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LiveText {
    tracer: BindedTracer<LiveTextState>,
}

impl LiveText {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        mode: FlowMode,
        spec: impl Into<LiveTextSpec>,
    ) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    pub fn set(&self, text: impl Into<String>) {
        let msg = LiveTextEvent::Set(text.into());
        self.tracer.send(msg, None);
    }
}
