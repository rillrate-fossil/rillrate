use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterState {
    pub spec: CounterSpec,
    pub total: f64,
}

impl From<CounterSpec> for CounterState {
    fn from(spec: CounterSpec) -> Self {
        Self { spec, total: 0.0 }
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
    Inc { delta: f64 },
}
