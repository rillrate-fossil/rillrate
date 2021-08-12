use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::ready_board::{Board, ReadyBoardEvent, ReadyBoardState};
use rill_protocol::io::provider::Path;
use std::collections::HashSet;

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ReadyBoardTracer {
    tracer: Tracer<ReadyBoardState>,
}

impl ReadyBoardTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let state = ReadyBoardState::new();
        let tracer = Tracer::new_push(state, path, None);
        Self { tracer }
    }

    /// Add a board
    pub fn add_board(&self, name: String, paths: HashSet<Path>, description: Option<String>) {
        let board = Board { description, paths };
        let data = ReadyBoardEvent::AddBoard { name, board };
        self.tracer.send(data, None);
    }
}
