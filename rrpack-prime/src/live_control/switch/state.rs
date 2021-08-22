use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchSpec {
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchState {
    pub spec: SwitchSpec,
    pub turned_on: bool,
}

impl From<SwitchSpec> for SwitchState {
    fn from(spec: SwitchSpec) -> Self {
        Self {
            spec,
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

pub type SwitchAction = bool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchEvent {
    pub turn_on: bool,
}
