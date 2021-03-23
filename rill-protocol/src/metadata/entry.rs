use crate::data::{Metric, TimedEvent};
use crate::io::provider::{EntryId, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntryMetric;

impl Metric for EntryMetric {
    type State = EntryState;
    type Event = EntryEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.entry.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            EntryEvent::AddProvider { name } => {
                state.providers.insert(name, ());
            }
            EntryEvent::RemoveProvider { name } => {
                state.providers.remove(&name);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryState {
    pub providers: BTreeMap<EntryId, ()>,
}

#[allow(clippy::new_without_default)]
impl EntryState {
    pub fn new() -> Self {
        Self {
            providers: BTreeMap::new(),
        }
    }
}

pub type EntryDelta = Vec<TimedEvent<EntryEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryEvent {
    AddProvider { name: EntryId },
    RemoveProvider { name: EntryId },
}
