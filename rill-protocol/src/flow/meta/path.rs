use crate::flow::core::{Flow, TimedEvent};
use crate::flow::locations::Location;
use crate::io::provider::{Description, Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PATHS: Location = Location::new("meta:paths");

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

impl Flow for PathState {
    type Action = ();
    type Event = PathEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.path.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            PathEvent::AddPath { path, description } => {
                self.entries.insert(path, description);
            }
            PathEvent::RemovePath { path } => {
                self.entries.remove(&path);
            }
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
