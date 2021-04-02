use super::MetaFlow;
use crate::flow::data::{Flow, TimedEvent};
use crate::io::provider::{EntryId, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EntryFlow;

impl MetaFlow for EntryFlow {}

impl Flow for EntryFlow {
    type State = EntryState;
    type Event = EntryEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.entry.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            EntryEvent::AddEntry { name } => {
                state.entries.insert(name, ());
            }
            EntryEvent::RemoveEntry { name } => {
                state.entries.remove(&name);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryState {
    pub entries: BTreeMap<EntryId, ()>,
}

#[allow(clippy::new_without_default)]
impl EntryState {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

pub type EntryDelta = Vec<TimedEvent<EntryEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryEvent {
    AddEntry { name: EntryId },
    RemoveEntry { name: EntryId },
}
