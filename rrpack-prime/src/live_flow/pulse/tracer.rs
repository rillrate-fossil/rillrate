use super::state::*;
use crate::auto_path::AutoPath;
use crate::base_flow::frame_flow::FrameFlowTracer;
use crate::manifest::Binded;

pub struct Pulse {
    tracer: Binded<FrameFlowTracer<PulseSpec>>,
}

impl Pulse {
    pub fn new(auto_path: impl Into<AutoPath>, spec: PulseSpec) -> Self {
        let path = auto_path.into();
        let tracer = Binded::new(FrameFlowTracer::new(path.into(), spec));
        Self { tracer }
    }

    pub fn push(&self, value: f64) {
        self.tracer.add_frame(value);
    }
}
