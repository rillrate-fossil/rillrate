use crate::manifest::description::{Layer, PackFlow};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterState {
    pub spec: CounterSpec,
    /// It's integer to be split in parts (mil, thous, ones).
    pub total: i64,
}

impl From<CounterSpec> for CounterState {
    fn from(spec: CounterSpec) -> Self {
        Self { spec, total: 0 }
    }
}

impl PackFlow for CounterState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for CounterState {
    type Action = ();
    type Event = CounterEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            CounterEvent::Inc { delta } => {
                self.total += delta;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Inc { delta: i64 },
}
