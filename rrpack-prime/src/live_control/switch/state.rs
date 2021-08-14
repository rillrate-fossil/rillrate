use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchState {
    pub label: String,
    pub turned_on: bool,
}

impl SwitchState {
    pub fn new(label: String) -> Self {
        Self {
            label,
            turned_on: false,
        }
    }
}

impl Flow for SwitchState {
    type Action = SwitchAction;
    type Event = SwitchEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        self.turned_on = event.turn_on;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchAction {
    pub turn_on: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchEvent {
    pub turn_on: bool,
}
