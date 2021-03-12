use super::{Metric, TimedEvent};
use crate::io::provider::Timestamp;
use serde::{Deserialize, Serialize};

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
