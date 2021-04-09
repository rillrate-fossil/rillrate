use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

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

impl Flow for CounterState {
    type Event = CounterEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.data.counter.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            CounterEvent::Inc(delta) => {
                self.timestamp = Some(event.timestamp);
                self.value += delta;
            }
        }
    }
}

pub type CounterDelta = Vec<TimedEvent<CounterEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Inc(f64),
}
