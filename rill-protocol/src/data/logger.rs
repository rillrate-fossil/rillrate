use super::{ConvertError, Delta, Event, State, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
    type Delta = LogDelta;
    type Event = LogEvent;

    fn apply(&mut self, update: Self::Delta) {
        for event in update {
            self.frame.insert(event);
        }
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

impl Delta for LogDelta {
    type Event = LogEvent;

    fn push(&mut self, event: TimedEvent<Self::Event>) {
        self.push(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    // TODO: Replace with enum
    pub msg: String,
}

impl Event for LogEvent {
    type State = LogState;
    type Delta = LogDelta;
}
