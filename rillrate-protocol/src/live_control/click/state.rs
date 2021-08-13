use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickState {
    pub caption: String,
}

impl ClickState {
    pub fn new(caption: String) -> Self {
        Self { caption }
    }
}

impl Flow for ClickState {
    type Action = ClickAction;
    type Event = ClickEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, _event: Self::Event) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;
