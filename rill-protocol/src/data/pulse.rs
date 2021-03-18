use super::{Metric, Pct, TimedEvent};
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
        // Use the clamped value if a range set, but don't affect the state.
        let mut value = state.value;
        if let Some(range) = state.range.as_ref() {
            range.clamp(&mut value);
        }
        let point = PulsePoint { value };
        let timed_event = TimedEvent {
            timestamp: event.timestamp,
            event: point,
        };
        state.frame.insert(timed_event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulsePoint {
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseState {
    pub range: Option<Range>,
    pub frame: Frame<TimedEvent<PulsePoint>>,
    /// Intermediate counter value. Not available for changing!!!
    value: f64,
}

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

    pub fn has_range(&self) -> bool {
        self.range.is_some()
    }

    pub fn has_window(&self) -> bool {
        self.frame.size() > 1
    }

    pub fn pct(&self) -> Pct {
        if let Some(range) = self.range.as_ref() {
            Pct::from_range(self.value, range)
        } else {
            // TODO: If frame exists than calculate in that range
            Pct::from_value(0.0)
        }
    }
}

pub type PulseDelta = Vec<TimedEvent<PulseEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PulseEvent {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}
