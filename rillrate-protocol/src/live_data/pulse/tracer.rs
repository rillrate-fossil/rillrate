use super::state::*;
use crate::base::frame_flow::{FrameFlowTracer, FrameFlowWatcher, FrameFlowSpec};
use crate::manifest::descriptions_flow::DescriptionBinder;
use rill_protocol::io::provider::EntryId;

pub type PulseFrameTracer = FrameFlowTracer<PulseFrameSpec>;

pub struct CounterStatTracer {
    tracer: FrameFlowTracer<PulseFrameSpec>,
    binder: DescriptionBinder,
}

impl CounterStatTracer {
    pub fn new(name: impl Into<EntryId>) -> Self {
        let spec = PulseFrameSpec {
            name: name.into(),
        };
        let path = spec.path();
        // TODO: Use `info` later for labels, scale, etc.
        let info = ();
        let tracer = FrameFlowTracer::new(path, info).0;
        let binder = DescriptionBinder::new(&tracer);
        Self { tracer, binder }
    }

    pub fn add(&self, value: f32) {
        self.tracer.add_frame(value);
    }
}
