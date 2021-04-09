use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToggleFlow;

impl Flow for ToggleFlow {
    type State = ToggleState;
    type Event = ToggleEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.toggle.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        state.active = event.event.active;
        state.last_toggle = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleState {
    // IMMUTABLE
    pub caption: String,

    // MUTABLE
    pub active: bool,
    pub last_toggle: Option<Timestamp>,
}

#[allow(clippy::new_without_default)]
impl ToggleState {
    pub fn new(caption: String, active: bool) -> Self {
        Self {
            caption,
            active,
            last_toggle: None,
        }
    }

    pub fn toggle_event(&self) -> ToggleEvent {
        ToggleEvent::new(!self.active)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleEvent {
    pub active: bool,
}

impl ToggleEvent {
    pub fn new(active: bool) -> Self {
        Self { active }
    }

    pub fn on() -> Self {
        Self { active: true }
    }

    pub fn off() -> Self {
        Self { active: false }
    }
}
