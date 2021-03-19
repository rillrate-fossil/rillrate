use super::{Metric, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use crate::range::Range;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GaugeMetric {
    pub range: Range,
}

impl Metric for GaugeMetric {
    type State = GaugeState;
    type Event = GaugeEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.gauge.v0")
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
    pub timestamp: Option<Timestamp>,
    pub value: f64,
}

impl GaugeState {
    pub fn new() -> Self {
        Self {
            timestamp: None,
            value: 0.0,
        }
    }

    pub fn value(&self) -> Option<(Timestamp, f64)> {
        self.timestamp.map(|ts| (ts, self.value))
    }

    /*
    pub fn pct(&self) -> Pct {
        Pct::from_range(self.value, &Range::new(self.min, self.max))
    }
    */
}

pub type GaugeDelta = Vec<TimedEvent<GaugeEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Set(f64),
}
