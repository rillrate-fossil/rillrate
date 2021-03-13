use super::{Metric, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct PulseMetric;

impl Metric for PulseMetric {
    type State = PulseState;
    type Event = PulseEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.pulse.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            PulseEvent::Increment(delta) => {
                state.value += delta;
            }
            PulseEvent::Decrement(delta) => {
                state.value -= delta;
            }
            PulseEvent::Set(value) => {
                state.value = value;
            }
        }
        let point = GaugePoint { value: state.value };
        let timed_event = TimedEvent {
            timestamp: event.timestamp,
            event: point,
        };
        state.frame.insert(timed_event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugePoint {
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseState {
    pub frame: Frame<TimedEvent<GaugePoint>>,
    value: f64,
}

impl PulseState {
    pub fn new() -> Self {
        Self {
            // TODO: Use duration for removing obsolete values instead
            frame: Frame::new(100),
            value: 0.0,
        }
    }
}

pub type GaugeDelta = Vec<TimedEvent<PulseEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PulseEvent {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}
