use super::{ConvertError, Delta, State, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugePoint {
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeState {
    pub frame: Frame<TimedEvent<GaugePoint>>,
    value: f64,
}

impl Default for GaugeState {
    fn default() -> Self {
        Self {
            // TODO: Use duration for removing obsolete values instead
            frame: Frame::new(100),
            value: 0.0,
        }
    }
}

impl TryFrom<StreamState> for GaugeState {
    type Error = ConvertError;

    fn try_from(state: StreamState) -> Result<Self, ConvertError> {
        match state {
            StreamState::Gauge(state) => Ok(state),
            _ => Err(ConvertError),
        }
    }
}

impl State for GaugeState {
    type Event = GaugeEvent;

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            GaugeEvent::Increment(delta) => {
                self.value += delta;
            }
            GaugeEvent::Decrement(delta) => {
                self.value -= delta;
            }
            GaugeEvent::Set(value) => {
                self.value = value;
            }
        }
        let point = GaugePoint { value: self.value };
        let timed_event = TimedEvent {
            timestamp: event.timestamp,
            event: point,
        };
        self.frame.insert(timed_event);
    }

    fn wrap(events: Delta<Self::Event>) -> StreamDelta {
        StreamDelta::from(events)
    }

    fn try_extract(delta: StreamDelta) -> Result<Delta<Self::Event>, ConvertError> {
        delta.try_into()
    }
}

pub type GaugeDelta = Vec<TimedEvent<GaugeEvent>>;

impl TryFrom<StreamDelta> for GaugeDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Gauge(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}
