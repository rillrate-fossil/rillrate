use super::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterFlow;

impl Flow for CounterFlow {
    type State = CounterState;
    type Event = CounterEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.data.counter.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            CounterEvent::Inc(delta) => {
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

    pub fn last(&self) -> Option<TimedEvent<f64>> {
        self.timestamp.map(|ts| TimedEvent {
            timestamp: ts,
            event: self.value,
        })
    }
}

pub type CounterDelta = Vec<TimedEvent<CounterEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Inc(f64),
}
