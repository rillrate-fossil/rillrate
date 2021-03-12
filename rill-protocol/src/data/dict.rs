use super::{Metric, TimedEvent};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DictMetric;

impl Metric for DictMetric {
    type State = DictState;
    type Event = DictEvent;

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            DictEvent::SetValue { key, value } => {
                state.map.insert(key, value);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictState {
    pub map: BTreeMap<String, String>,
}

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
    SetValue { key: String, value: String },
}
