use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::timed;
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Pulse {
    tracer: BindedTracer<PulseState>,
}

impl Pulse {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: impl Into<PulseSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    pub fn push(&self, value: f64) {
        if let Some(value) = timed(value) {
            let msg = PulseEvent::Push { value };
            self.tracer.send(msg, None);
        }
    }
}
