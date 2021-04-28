use crate::flow::core::{Flow, TimedEvent, ToEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

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

    pub fn toggle_action(&self) -> ToggleAction {
        ToggleAction::new(!self.active)
    }
}

impl Flow for ToggleState {
    type Action = ToggleAction;
    type Event = ToggleEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.toggle.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        self.active = event.event.active;
        self.last_toggle = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleAction {
    pub set_active: bool,
}

impl ToggleAction {
    pub fn new(set_active: bool) -> Self {
        Self { set_active }
    }

    pub fn on() -> Self {
        Self { set_active: true }
    }

    pub fn off() -> Self {
        Self { set_active: false }
    }
}

impl ToEvent<ToggleEvent> for ToggleAction {
    fn to_event(&self) -> Option<ToggleEvent> {
        Some(ToggleEvent {
            active: self.set_active,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleEvent {
    pub active: bool,
}
