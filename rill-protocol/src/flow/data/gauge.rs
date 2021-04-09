use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use crate::range::Range;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GaugeFlow;

impl Flow for GaugeFlow {
    type State = GaugeState;
    type Event = GaugeEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.data.gauge.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            GaugeEvent::Set(delta) => {
                state.timestamp = Some(event.timestamp);
                state.value = delta;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeState {
    // IMMUTABLE:
    pub range: Range,

    // MUTABLE:
    pub timestamp: Option<Timestamp>,
    pub value: f64,
}

impl GaugeState {
    pub fn new(range: Range) -> Self {
        Self {
            range,
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

pub type GaugeDelta = Vec<TimedEvent<GaugeEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Set(f64),
}
