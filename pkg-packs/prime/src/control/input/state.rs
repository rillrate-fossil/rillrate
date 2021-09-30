use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use rrpack_basis::manifest::description::{Layer, PackFlow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    pub label: String,
    pub wide: bool,
    pub password: bool,
    pub placeholder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputState {
    pub spec: InputSpec,
    pub text: String,
}

impl From<InputSpec> for InputState {
    fn from(spec: InputSpec) -> Self {
        Self {
            spec,
            text: String::new(),
        }
    }
}

impl PackFlow for InputState {
    fn layer() -> Layer {
        Layer::Control
    }
}

impl Flow for InputState {
    type Action = InputAction;
    type Event = InputEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        self.text = event.changed_text;
    }
}

pub type InputAction = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub changed_text: String,
}
