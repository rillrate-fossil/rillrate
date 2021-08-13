use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorState {
    pub label: String,
    pub options: Vec<String>,
    pub selected: Option<String>,
}

impl SelectorState {
    pub fn new(label: String, options: Vec<String>) -> Self {
        Self {
            label,
            options,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorAction {
    pub new_selected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorEvent {
    pub update_selected: Option<String>,
}
