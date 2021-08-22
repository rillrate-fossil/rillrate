use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorSpec {
    pub label: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorState {
    pub spec: SelectorSpec,
    pub selected: Option<String>,
}

impl From<SelectorSpec> for SelectorState {
    fn from(spec: SelectorSpec) -> Self {
        Self {
            spec,
            selected: None,
        }
    }
}

impl Flow for SelectorState {
    type Action = SelectorAction;
    type Event = SelectorEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        self.selected = event.update_selected;
    }
}

pub type SelectorAction = Option<String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorEvent {
    pub update_selected: Option<String>,
}
