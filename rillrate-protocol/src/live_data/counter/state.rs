use rill_protocol::flow::core::{DataFraction, Flow};
use rill_protocol::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterState {
    total: i64,
}

#[allow(clippy::new_without_default)]
impl CounterState {
    pub fn new() -> Self {
        Self { total: 0 }
    }
}

impl Flow for CounterState {
    type Action = CounterAction;
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
pub enum CounterAction {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterEvent {
    Inc { delta: i64 },
}
