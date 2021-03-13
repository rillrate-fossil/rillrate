use super::{Metric, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct LogMetric;

impl Metric for LogMetric {
    type State = LogState;
    type Event = LogEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.logger.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        state.frame.insert(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogState {
    pub frame: Frame<TimedEvent<LogEvent>>,
}

impl LogState {
    pub fn new() -> Self {
        Self {
            frame: Frame::new(10),
        }
    }
}

pub type LogDelta = Vec<TimedEvent<LogEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    // TODO: Replace with enum
    pub msg: String,
}
