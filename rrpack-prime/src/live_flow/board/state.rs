use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
    pub map: BTreeMap<String, String>,
}

#[allow(clippy::new_without_default)]
impl BoardState {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
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
