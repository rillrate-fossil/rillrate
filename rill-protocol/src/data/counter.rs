use super::{ConvertError, Delta, Metric, TimedEvent};
use crate::io::provider::{StreamDelta, StreamState, Timestamp};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct CounterMetric;

impl Metric for CounterMetric {
    type State = CounterState;
    type Event = CounterEvent;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            CounterEvent::Increment(delta) => {
                state.timestamp = Some(event.timestamp);
                state.value += delta;
            }
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Increment(f64),
}
