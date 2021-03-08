use super::{Delta, Event, State, TimedEvent};
use crate::frame::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogState {
    frame: Frame<TimedEvent<LogEvent>>,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            frame: Frame::new(30),
        }
    }
}

impl State for LogState {
    type Delta = LogDelta;

    fn apply(&mut self, update: Self::Delta) {
        for event in update.events {
            self.frame.insert(event);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDelta {
    events: Vec<TimedEvent<LogEvent>>,
}

impl Delta for LogDelta {
    type Event = LogEvent;

    fn produce(event: TimedEvent<Self::Event>) -> Self {
        Self {
            events: vec![event],
        }
    }

    fn combine(&mut self, event: TimedEvent<Self::Event>) {
        self.events.push(event);
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
