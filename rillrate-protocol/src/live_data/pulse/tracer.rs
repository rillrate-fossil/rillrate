use super::state::*;
use crate::base::frame_flow::FrameFlowTracer;
use crate::manifest::Binded;
use rill_protocol::io::provider::EntryId;

pub struct PulseFrameTracer {
    tracer: Binded<FrameFlowTracer<PulseFrameSpec>>,
}

impl PulseFrameTracer {
    pub fn new(group: EntryId, name: EntryId) -> Self {
        let path = vec![group, name].into();
        // TODO: Use `info` later for labels, scale, etc.
        let info = ();
        let tracer = Binded::new(FrameFlowTracer::new(path, info).0);
        Self { tracer }
    }

    pub fn add(&self, value: f32) {
        self.tracer.add_frame(value);
    }
}
