use super::MetaFlow;
use crate::flow::data::{Flow, TimedEvent};
use crate::io::codec::vectorize;
use crate::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReadyBoardFlow;

impl MetaFlow for ReadyBoardFlow {
    fn location() -> Path {
        Path::single("meta:readyboards")
    }
}

impl Flow for ReadyBoardFlow {
    type State = ReadyBoardState;
    type Event = ReadyBoardEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.readyboard.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            ReadyBoardEvent::AddBoard { name, paths } => {
                state.entries.insert(name, paths);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyBoardState {
    // Vectorizing is not necessary here, but useful
    // if key type will be changed to another type.
    #[serde(with = "vectorize")]
    pub entries: BTreeMap<String, Vec<Path>>,
}

#[allow(clippy::new_without_default)]
impl ReadyBoardState {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

pub type ReadyBoardDelta = Vec<TimedEvent<ReadyBoardEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadyBoardEvent {
    AddBoard { name: String, paths: Vec<Path> },
}
