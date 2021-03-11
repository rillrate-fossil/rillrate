use super::{ConvertError, Delta, Event, State, TimedEvent};
use crate::io::provider::{StreamDelta, StreamState, Timestamp};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterState {
    pub timestamp: Option<Timestamp>,
    pub value: f64,
}

impl Default for CounterState {
    fn default() -> Self {
        Self {
            timestamp: None,
            value: 0.0,
        }
    }
}

impl TryFrom<StreamState> for CounterState {
    type Error = ConvertError;

    fn try_from(state: StreamState) -> Result<Self, ConvertError> {
        match state {
            StreamState::Counter(state) => Ok(state),
            _ => Err(ConvertError),
        }
    }
}

impl State for CounterState {
    type Delta = CounterDelta;
    type Event = CounterEvent;

    fn apply(&mut self, delta: Self::Delta) {
        for event in delta {
            match event.event {
                CounterEvent::Increment(delta) => {
                    self.timestamp = Some(event.timestamp);
                    self.value += delta;
                }
            }
        }
    }
}

pub type CounterDelta = Vec<TimedEvent<CounterEvent>>;

impl TryFrom<StreamDelta> for CounterDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Counter(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

impl Delta for CounterDelta {
    type Event = CounterEvent;

    fn produce(event: TimedEvent<Self::Event>) -> Self {
        vec![event]
    }

    fn combine(&mut self, event: TimedEvent<Self::Event>) {
        self.push(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Increment(f64),
}

impl Event for CounterEvent {
    type State = CounterState;
    type Delta = CounterDelta;
}
