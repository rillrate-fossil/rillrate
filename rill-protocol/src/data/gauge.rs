use super::{Delta, Event, State, TimedEvent, Timestamp};
use crate::frame::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugePoint {
    pub timestamp: Timestamp,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeState {
    pub frame: Frame<GaugePoint>,
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
            let point = GaugePoint {
                timestamp: event.timestamp,
                value: self.value,
            };
            self.frame.insert(point);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeDelta {
    events: Vec<TimedEvent<GaugeEvent>>,
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
