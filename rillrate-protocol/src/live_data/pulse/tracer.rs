use super::state::*;
use crate::base::frame_flow::FrameFlowTracer;
use crate::live_data::live_path::LivePath;
use crate::manifest::Binded;

pub struct PulseFrameTracer {
    tracer: Binded<FrameFlowTracer<PulseFrameSpec>>,
}

impl PulseFrameTracer {
    pub fn new(live_path: LivePath) -> Self {
        let path = live_path.into();
        // TODO: Use `info` later for labels, scale, etc.
        let info = ();
        let tracer = Binded::new(FrameFlowTracer::new(path, info).0);
        Self { tracer }
    }

    pub fn add(&self, value: f32) {
        self.tracer.add_frame(value);
    }
}
