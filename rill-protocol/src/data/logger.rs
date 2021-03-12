use super::{ConvertError, Metric, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::StreamState;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct LogMetric;

impl Metric for LogMetric {
    type State = LogState;
    type Event = LogEvent;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        state.frame.insert(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogState {
    pub frame: Frame<TimedEvent<LogEvent>>,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            frame: Frame::new(10),
        }
    }
}

impl TryFrom<StreamState> for LogState {
    type Error = ConvertError;

    fn try_from(state: StreamState) -> Result<Self, ConvertError> {
        match state {
            StreamState::Log(state) => Ok(state),
            _ => Err(ConvertError),
        }
    }
}

pub type LogDelta = Vec<TimedEvent<LogEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    // TODO: Replace with enum
    pub msg: String,
}
