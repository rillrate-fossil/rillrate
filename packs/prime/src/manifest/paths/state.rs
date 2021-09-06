use crate::manifest::description::PackFlowDescription;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

impl PathsSpec {
    pub fn path() -> Path {
        "rillrate.manifest.descriptions_list".parse().unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsState {
    #[serde(with = "vectorize")]
    pub records: BTreeMap<Path, PackFlowDescription>,
}

impl From<PathsSpec> for PathsState {
    fn from(_spec: PathsSpec) -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl Flow for PathsState {
    type Action = ();
    type Event = PathsEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            PathsEvent::Add { path, description } => {
                self.records.insert(path, description);
            }
            PathsEvent::Remove { path } => {
                self.records.remove(&path);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathsEvent {
    Add {
        path: Path,
        description: PackFlowDescription,
    },
    Remove {
        path: Path,
    },
}
