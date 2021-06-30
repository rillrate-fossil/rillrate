use crate::flow::core::Flow;
use crate::flow::location::Location;
use crate::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const READY_BOARDS: Location = Location::new("meta:readyboards");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub description: Option<String>,
    pub paths: HashSet<Path>,
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

impl Flow for ReadyBoardState {
    type Action = ();
    type Event = ReadyBoardEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.meta.readyboard.v0")
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ReadyBoardEvent::AddBoard { name, board } => {
                self.entries.insert(name, board);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadyBoardEvent {
    AddBoard { name: String, board: Board },
}
