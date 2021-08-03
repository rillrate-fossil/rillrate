use super::state::*;
use crate::base::frame_flow::{FrameFlowSpec, FrameFlowTracer, FrameFlowWatcher};
use crate::manifest::descriptions_flow::Binded;
use rill_protocol::io::provider::EntryId;

pub type PulseFrameTracer = FrameFlowTracer<PulseFrameSpec>;

pub struct CounterStatTracer {
    tracer: Binded<FrameFlowTracer<PulseFrameSpec>>,
}

impl CounterStatTracer {
    pub fn new(name: impl Into<EntryId>) -> Self {
        let spec = PulseFrameSpec { name: name.into() };
        let path = spec.path();
        // TODO: Use `info` later for labels, scale, etc.
        let info = ();
        let tracer = Binded::new(FrameFlowTracer::new(path, info).0);
        Self { tracer }
    }

    pub fn add(&self, value: f32) {
        self.tracer.add_frame(value);
    }
}
