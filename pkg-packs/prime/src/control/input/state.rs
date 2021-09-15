use crate::manifest::description::{Layer, PackFlow};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputState {
    pub spec: InputSpec,
}

impl From<InputSpec> for InputState {
    fn from(spec: InputSpec) -> Self {
        Self { spec }
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

    fn apply(&mut self, _event: Self::Event) {}
}

pub type InputAction = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent;
