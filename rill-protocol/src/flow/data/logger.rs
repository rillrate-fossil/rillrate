use crate::flow::core::{Flow, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

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

impl Flow for LoggerState {
    type Event = LoggerEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.data.logger.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        self.frame.insert(event);
    }
}

pub type LoggerDelta = Vec<TimedEvent<LoggerEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerEvent {
    // TODO: Replace with enum
    pub msg: String,
}
