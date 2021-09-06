use super::layout::Layout;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Path, EntryId, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

impl LayoutsSpec {
    pub fn path() -> Path {
        "rillrate.manifest.layouts".parse().unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutsSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutsState {
    #[serde(with = "vectorize")]
    pub records: BTreeMap<EntryId, Layout>,
}

impl From<LayoutsSpec> for LayoutsState {
    fn from(_spec: LayoutsSpec) -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl Flow for LayoutsState {
    type Action = ();
    type Event = LayoutsEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            LayoutsEvent::Add { name, layout } => {
                self.records.insert(name, layout);
            }
            LayoutsEvent::Remove { name } => {
                self.records.remove(&name);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutsEvent {
    Add {
        name: EntryId,
        layout: Layout,
    },
    Remove {
        name: EntryId,
    },
}
