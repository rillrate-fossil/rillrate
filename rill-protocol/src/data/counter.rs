use super::{Metric, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CounterMetric;

impl Metric for CounterMetric {
    type State = CounterState;
    type Event = CounterEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.counter.v0")
    }

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

#[allow(clippy::new_without_default)]
impl CounterState {
    pub fn new() -> Self {
        Self {
            timestamp: None,
            value: 0.0,
        }
    }
}

pub type CounterDelta = Vec<TimedEvent<CounterEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Increment(f64),
}
