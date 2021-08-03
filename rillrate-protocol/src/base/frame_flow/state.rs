use crate::base::new_tf;
use rill_protocol::flow::core::{DataFraction, Flow, TimedEvent};
use rill_protocol::io::provider::{Path, StreamType};
use rill_protocol::timed_frame::TimedFrame;
use serde::{Deserialize, Serialize};

pub trait FrameFlowSpec: DataFraction {
    type Info: DataFraction;
    type Frame: DataFraction;

    fn path(&self) -> Path;

    fn retain_secs() -> u32;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameFlowState<T: FrameFlowSpec> {
    pub info: T::Info,
    pub frame: TimedFrame<T::Frame>,
}

#[allow(clippy::new_without_default)]
impl<T: FrameFlowSpec> FrameFlowState<T> {
    pub fn new(info: T::Info) -> Self {
        Self {
            info,
            frame: new_tf(T::retain_secs() as i64),
        }
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
    // UpdateInfo { info }
    AddFrame { event: TimedEvent<T::Frame> },
}
