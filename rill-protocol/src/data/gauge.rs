use super::{Metric, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GaugeMetric;

impl Metric for GaugeMetric {
    type State = GaugeState;
    type Event = GaugeEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.gauge.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
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
    pub min: f64,
    pub max: f64,
    pub timestamp: Option<Timestamp>,
    pub value: f64,
}

impl GaugeState {
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            timestamp: None,
            value: 0.0,
        }
    }
}

pub type GaugeDelta = Vec<TimedEvent<GaugeEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Set(f64),
}
