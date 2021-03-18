use super::{Metric, TimedEvent};
// TODO: Join `Frame` and `Range` into a single module.
use crate::frame::Frame;
use crate::io::provider::StreamType;
use crate::range::Range;
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
        if let Some(range) = state.range.as_ref() {
            range.clamp(&mut state.value);
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
    pub range: Option<Range>,
    pub frame: Frame<TimedEvent<GaugePoint>>,
    value: f64,
}

#[allow(clippy::new_without_default)]
impl PulseState {
    pub fn new(range: Option<Range>, depth: Option<u32>) -> Self {
        let mut depth = depth.unwrap_or_default();
        if depth == 0 {
            depth = 1;
        }
        Self {
            // TODO: Use duration for removing obsolete values instead
            range: range,
            frame: Frame::new(depth),
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
