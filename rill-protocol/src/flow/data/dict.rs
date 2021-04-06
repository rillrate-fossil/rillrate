use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictFlow;

impl Flow for DictFlow {
    type State = DictState;
    type Event = DictEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.data.dict.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            DictEvent::Assign { key, value } => {
                state.map.insert(key, value);
            }
            DictEvent::Remove { key } => {
                state.map.remove(&key);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictState {
    pub map: BTreeMap<String, String>,
}

#[allow(clippy::new_without_default)]
impl DictState {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

pub type DictDelta = Vec<TimedEvent<DictEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DictEvent {
    Assign { key: String, value: String },
    Remove { key: String },
}
