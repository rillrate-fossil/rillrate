use super::MetaFlow;
use crate::flow::data::{Flow, TimedEvent};
use crate::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PathFlow;

impl MetaFlow for PathFlow {
    fn location() -> Path {
        Path::single("meta:paths")
    }
}

impl Flow for PathFlow {
    type State = PathState;
    type Event = PathEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.path.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            PathEvent::AddPath { name } => {
                state.entries.insert(name, ());
            }
            PathEvent::RemovePath { name } => {
                state.entries.remove(&name);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathState {
    pub entries: BTreeMap<Path, ()>,
}

#[allow(clippy::new_without_default)]
impl PathState {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

pub type PathDelta = Vec<TimedEvent<PathEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathEvent {
    AddPath { name: Path },
    RemovePath { name: Path },
}
