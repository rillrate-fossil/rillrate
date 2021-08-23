use crate::manifest::description::{Layer, PackFlow};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoardSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
    pub spec: BoardSpec,
    pub map: BTreeMap<String, String>,
}

impl From<BoardSpec> for BoardState {
    fn from(spec: BoardSpec) -> Self {
        Self {
            spec,
            map: BTreeMap::new(),
        }
    }
}

impl PackFlow for BoardState {
    fn layer() -> Layer {
        Layer::Visual
    }
}

impl Flow for BoardState {
    type Action = ();
    type Event = BoardEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            BoardEvent::Assign { key, value } => {
                self.map.insert(key, value);
            }
            BoardEvent::Remove { key } => {
                self.map.remove(&key);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardEvent {
    Assign { key: String, value: String },
    Remove { key: String },
}
