use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Description, Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

impl DescriptionsListSpec {
    pub fn path() -> Path {
        "rillrate.manifest.descriptions_list".parse().unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionsListSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionsListState {
    #[serde(with = "vectorize")]
    pub records: BTreeMap<Path, Description>,
}

impl From<DescriptionsListSpec> for DescriptionsListState {
    fn from(_spec: DescriptionsListSpec) -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl Flow for DescriptionsListState {
    type Action = ();
    type Event = DescriptionsListEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            DescriptionsListEvent::Add { path, description } => {
                self.records.insert(path, description);
            }
            DescriptionsListEvent::Remove { path } => {
                self.records.remove(&path);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DescriptionsListEvent {
    Add {
        path: Path,
        description: Description,
    },
    Remove {
        path: Path,
    },
}
