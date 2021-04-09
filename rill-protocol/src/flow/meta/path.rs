use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{Description, Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PathFlow;

impl Flow for PathFlow {
    type State = PathState;
    type Event = PathEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.path.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            PathEvent::AddPath { path, description } => {
                state.entries.insert(path, description);
            }
            PathEvent::RemovePath { path } => {
                state.entries.remove(&path);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathState {
    #[serde(with = "vectorize")]
    pub entries: BTreeMap<Path, Description>,
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
    AddPath {
        path: Path,
        description: Description,
    },
    RemovePath {
        path: Path,
    },
}
