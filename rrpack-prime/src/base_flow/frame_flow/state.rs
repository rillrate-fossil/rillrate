use crate::base_flow::new_tf;
use rill_protocol::flow::core::{DataFraction, Flow, TimedEvent};
use rill_protocol::io::provider::StreamType;
use rill_protocol::timed_frame::TimedFrame;
use serde::{Deserialize, Serialize};

pub trait FrameFlowSpec: DataFraction {
    type Frame: DataFraction;

    fn retain_secs(&self) -> u32;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameFlowState<T: FrameFlowSpec> {
    #[serde(bound = "")]
    pub spec: T,
    pub frame: TimedFrame<T::Frame>,
}

#[allow(clippy::new_without_default)]
impl<T: FrameFlowSpec> FrameFlowState<T> {
    pub fn new(spec: T) -> Self {
        let frame = new_tf(spec.retain_secs() as i64 + 1);
        Self { spec, frame }
    }
}

impl<T: FrameFlowSpec> Flow for FrameFlowState<T> {
    type Action = FrameFlowAction;
    type Event = FrameFlowEvent<T>;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            FrameFlowEvent::AddFrame { event } => {
                self.frame.insert_pop(event);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameFlowAction {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameFlowEvent<T: FrameFlowSpec> {
    // TODO: Maybe:
    // UpdateInfo { spec }
    AddFrame { event: TimedEvent<T::Frame> },
}