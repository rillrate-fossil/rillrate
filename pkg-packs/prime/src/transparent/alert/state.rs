use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use rrpack_basis::manifest::description::{Layer, PackFlow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSpec {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertState {
    // TODO: Decide: persistent or not?
}

impl From<AlertSpec> for AlertState {
    fn from(_spec: AlertSpec) -> Self {
        Self {}
    }
}

impl PackFlow for AlertState {
    fn layer() -> Layer {
        Layer::Transparent
    }
}

impl Flow for AlertState {
    type Action = AlertAction;
    type Event = AlertEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, _event: Self::Event) {}
}

pub type AlertAction = ();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertEvent {
    Notify { text: String },
}
