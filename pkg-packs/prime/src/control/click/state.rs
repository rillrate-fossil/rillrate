use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use rrpack_basis::manifest::description::{Layer, PackFlow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickSpec {
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickState {
    pub spec: ClickSpec,
}

impl From<ClickSpec> for ClickState {
    fn from(spec: ClickSpec) -> Self {
        Self { spec }
    }
}

impl PackFlow for ClickState {
    fn layer() -> Layer {
        Layer::Control
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

pub type ClickAction = ();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;
