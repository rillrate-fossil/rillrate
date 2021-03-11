use super::{ConvertError, Delta, State, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

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

impl State for LogState {
    type Event = LogEvent;

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        self.frame.insert(event);
    }

    fn wrap(events: Delta<Self::Event>) -> StreamDelta {
        StreamDelta::from(events)
    }

    fn try_extract(delta: StreamDelta) -> Result<Delta<Self::Event>, ConvertError> {
        delta.try_into()
    }
}

pub type LogDelta = Vec<TimedEvent<LogEvent>>;

impl TryFrom<StreamDelta> for LogDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Log(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    // TODO: Replace with enum
    pub msg: String,
}
