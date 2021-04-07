use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{StreamType, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToggleFlow {
    pub caption: String,
}

impl Flow for ToggleFlow {
    type State = ToggleState;
    type Event = ToggleEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.flow.control.toggle.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        let ToggleEvent::Set(new_value) = event.event;
        state.value = new_value;
        state.last_toggle = Some(event.timestamp);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleState {
    pub value: bool,
    pub last_toggle: Option<Timestamp>,
}

#[allow(clippy::new_without_default)]
impl ToggleState {
    pub fn new(value: bool) -> Self {
        Self {
            value,
            last_toggle: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToggleEvent {
    Set(bool),
}
