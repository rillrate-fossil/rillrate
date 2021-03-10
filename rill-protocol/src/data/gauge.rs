use super::{ConvertError, Delta, Event, State, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::{StreamDelta, StreamState};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
            frame: Frame::new(30),
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
    type Delta = GaugeDelta;

    fn apply(&mut self, delta: Self::Delta) {
        for event in delta.events {
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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeDelta {
    events: Vec<TimedEvent<GaugeEvent>>,
}

impl TryFrom<StreamDelta> for GaugeDelta {
    type Error = ConvertError;

    fn try_from(delta: StreamDelta) -> Result<Self, ConvertError> {
        match delta {
            StreamDelta::Gauge(delta) => Ok(delta),
            _ => Err(ConvertError),
        }
    }
}

impl Delta for GaugeDelta {
    type Event = GaugeEvent;

    fn produce(event: TimedEvent<Self::Event>) -> Self {
        Self {
            events: vec![event],
        }
    }

    fn combine(&mut self, event: TimedEvent<Self::Event>) {
        self.events.push(event);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}

impl Event for GaugeEvent {
    type State = GaugeState;
    type Delta = GaugeDelta;
}
