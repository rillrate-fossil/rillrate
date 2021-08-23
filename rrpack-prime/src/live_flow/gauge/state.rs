use crate::manifest::description::{Layer, PackFlow};
use crate::range::Range;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GaugeSpec {
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeState {
    pub spec: GaugeSpec,
    pub value: Option<f64>,
    pub abs_min: f64,
    pub abs_max: f64,
}

impl From<GaugeSpec> for GaugeState {
    fn from(spec: GaugeSpec) -> Self {
        Self {
            spec,
            value: None,
            abs_min: f64::MAX,
            abs_max: f64::MIN,
        }
    }
}

impl PackFlow for GaugeState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for GaugeState {
    type Action = ();
    type Event = GaugeEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            GaugeEvent::Set { value } => {
                self.value = Some(value);
                if value < self.abs_min {
                    self.abs_min = value;
                }
                if value > self.abs_max {
                    self.abs_max = value;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Set { value: f64 },
}
