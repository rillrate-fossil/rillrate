use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{EntryId, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

impl Flow for EntryState {
    type Event = EntryEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.entry.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            EntryEvent::AddEntry { name } => {
                self.entries.insert(name, ());
            }
            EntryEvent::RemoveEntry { name } => {
                self.entries.remove(&name);
            }
        }
    }
}

pub type EntryDelta = Vec<TimedEvent<EntryEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryEvent {
    AddEntry { name: EntryId },
    RemoveEntry { name: EntryId },
}
