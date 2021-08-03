use super::state::*;
use crate::base::frame_flow::{FrameFlowSpec, FrameFlowTracer};
use crate::manifest::descriptions_list::Binded;
use rill_protocol::io::provider::EntryId;

pub struct PulseFrameTracer {
    tracer: Binded<FrameFlowTracer<PulseFrameSpec>>,
}

impl PulseFrameTracer {
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
