use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{timed, Tracer};
use rill_protocol::io::provider::Path;

/// It records time automatically.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct FrameFlowTracer<T: FrameFlowSpec> {
    tracer: Tracer<FrameFlowState<T>>,
}

impl<T: FrameFlowSpec> FrameFlowTracer<T> {
    pub fn new(path: Path, spec: T) -> Self {
        let state = FrameFlowState::new(spec);
        let tracer = Tracer::new_push(state, path);
        Self { tracer }
    }

    pub fn add_frame(&self, frame: T::Frame) {
        if let Some(event) = timed(frame) {
            let msg = FrameFlowEvent::AddFrame { event };
            self.tracer.send(msg, None);
        }
    }
}
