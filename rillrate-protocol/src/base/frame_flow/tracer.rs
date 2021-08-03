use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::{timed, Tracer, Watcher};
use rill_protocol::io::provider::Path;

pub type FrameFlowWatcher<T> = Watcher<FrameFlowState<T>>;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct FrameFlowTracer<T: FrameFlowSpec> {
    tracer: Tracer<FrameFlowState<T>>,
}

impl<T: FrameFlowSpec> FrameFlowTracer<T> {
    pub fn new(path: Path, info: T::Info) -> (Self, FrameFlowWatcher<T>) {
        let state = FrameFlowState::new(info);
        let (tracer, watcher) = Tracer::new_push(state, path);
        (Self { tracer }, watcher)
    }

    pub fn add_frame(&self, frame: T::Frame) {
        if let Some(event) = timed(frame) {
            let msg = FrameFlowEvent::AddFrame { event };
            self.tracer.send(msg, None);
        }
    }
}
