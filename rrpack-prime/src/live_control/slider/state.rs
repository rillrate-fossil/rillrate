use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderState {
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: f64,
}

impl SliderState {
    pub fn new(label: String, min: f64, max: f64, step: f64) -> Self {
        Self {
            label,
            min,
            max,
            step,
            value: min,
        }
    }
}

impl Flow for SliderState {
    type Action = SliderAction;
    type Event = SliderEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        self.value = event.set_value.clamp(self.min, self.max);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderAction {
    pub new_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderEvent {
    pub set_value: f64,
}
