use super::layout::LayoutTab;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Path, StreamType};
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
    pub layouts: BTreeMap<Path, LayoutTab>,
}

impl From<LayoutsSpec> for LayoutsState {
    fn from(_spec: LayoutsSpec) -> Self {
        Self {
            layouts: BTreeMap::new(),
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
                self.layouts.insert(name, layout);
            }
            LayoutsEvent::Remove { name } => {
                self.layouts.remove(&name);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutsEvent {
    Add { name: Path, layout: LayoutTab },
    Remove { name: Path },
}
