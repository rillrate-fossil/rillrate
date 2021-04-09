use crate::flow::core::{Flow, TimedEvent};
use crate::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub description: Option<String>,
    pub paths: HashSet<Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReadyBoardFlow;

impl Flow for ReadyBoardFlow {
    type State = ReadyBoardState;
    type Event = ReadyBoardEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.readyboard.v0")
    }

    fn apply(&self, state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            ReadyBoardEvent::AddBoard { name, board } => {
                state.entries.insert(name, board);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyBoardState {
    // Vectorizing is not necessary here, but useful
    // if key type will be changed to another type.
    #[serde(with = "vectorize")]
    pub entries: BTreeMap<String, Board>,
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
    AddBoard { name: String, board: Board },
}
