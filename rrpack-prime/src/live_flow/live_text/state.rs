use crate::manifest::description::{Layer, PackFlow};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

pub struct LiveTextSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTextState {
    pub text: String,
}

impl From<LiveTextSpec> for LiveTextState {
    fn from(_spec: LiveTextSpec) -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl PackFlow for LiveTextState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for LiveTextState {
    type Action = ();
    type Event = LiveTextEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            LiveTextEvent::Set(new_text) => {
                self.text = new_text;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveTextEvent {
    Set(String),
}
