use crate::flow::core::Flow;
use crate::flow::location::Location;
use crate::io::provider::{Description, Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PATHS: Location = Location::new("meta:paths");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathState {
    /// Description of the provider
    pub description: Description,
    #[serde(with = "vectorize")]
    pub paths: BTreeMap<Path, Description>,
}

#[allow(clippy::new_without_default)]
impl PathState {
    pub fn new(description: Description) -> Self {
        Self {
            description,
            paths: BTreeMap::new(),
        }
    }
}

impl Flow for PathState {
    type Action = ();
    type Event = PathEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate::meta::path::v0")
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            PathEvent::AddPath { path, description } => {
                self.paths.insert(path, description);
            }
            PathEvent::RemovePath { path } => {
                self.paths.remove(&path);
            }
        }
    }
}

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
