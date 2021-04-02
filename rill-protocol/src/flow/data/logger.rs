use super::{Flow, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoggerFlow;

impl Flow for LoggerFlow {
    type State = LoggerState;
    type Event = LoggerEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.data.logger.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        state.frame.insert(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerState {
    pub frame: Frame<TimedEvent<LoggerEvent>>,
}

#[allow(clippy::new_without_default)]
impl LoggerState {
    pub fn new() -> Self {
        Self {
            frame: Frame::new(10),
        }
    }
}

pub type LoggerDelta = Vec<TimedEvent<LoggerEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerEvent {
    // TODO: Replace with enum
    pub msg: String,
}
