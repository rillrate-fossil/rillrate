use crate::range::{Label, Range};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use rrpack_basis::frames::{new_tf, TimedEvent, TimedFrame};
use rrpack_basis::manifest::description::{Layer, PackFlow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseSpec {
    pub retain: u32,
    pub range: Range,
    pub label: Label,
}

impl Default for PulseSpec {
    fn default() -> Self {
        Self {
            retain: 30,
            range: Range::default(),
            label: Label::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseState {
    pub spec: PulseSpec,
    pub frame: TimedFrame<f64>,
}

impl From<PulseSpec> for PulseState {
    fn from(spec: PulseSpec) -> Self {
        let frame = new_tf(spec.retain as i64 + 1);
        Self { spec, frame }
    }
}

impl PackFlow for PulseState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for PulseState {
    type Action = ();
    type Event = PulseEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            PulseEvent::Push { value } => {
                self.frame.insert_pop(value);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PulseEvent {
    Push { value: TimedEvent<f64> },
}
