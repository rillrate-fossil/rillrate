use crate::manifest::description::{Layer, PackFlow};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderSpec {
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderState {
    pub spec: SliderSpec,
    pub value: f64,
}

impl From<SliderSpec> for SliderState {
    fn from(spec: SliderSpec) -> Self {
        let value = spec.min;
        Self { spec, value }
    }
}

impl PackFlow for SliderState {
    fn layer() -> Layer {
        Layer::Control
    }
}

impl Flow for SliderState {
    type Action = SliderAction;
    type Event = SliderEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        self.value = event.set_value.clamp(self.spec.min, self.spec.max);
    }
}

pub type SliderAction = f64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderEvent {
    pub set_value: f64,
}
