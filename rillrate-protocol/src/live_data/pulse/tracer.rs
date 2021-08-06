use super::state::*;
use crate::base::frame_flow::FrameFlowTracer;
use crate::live_data::auto_path::AutoPath;
use crate::manifest::Binded;

pub struct PulseFrameTracer {
    tracer: Binded<FrameFlowTracer<PulseFrameSpec>>,
}

impl PulseFrameTracer {
    pub fn new(auto_path: AutoPath, spec: Option<PulseFrameSpec>) -> Self {
        let spec = spec.unwrap_or_default();
        let path = auto_path.into();
        let tracer = Binded::new(FrameFlowTracer::new(path, spec).0);
        Self { tracer }
    }

    pub fn add(&self, value: f32) {
        self.tracer.add_frame(value);
    }
}
