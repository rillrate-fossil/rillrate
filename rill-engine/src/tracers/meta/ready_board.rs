use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::{
    ready_board::{ReadyBoardEvent, ReadyBoardFlow, ReadyBoardState},
    MetaFlow,
};
use rill_protocol::io::provider::Path;
use std::collections::HashSet;

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ReadyBoardTracer {
    tracer: Tracer<ReadyBoardFlow>,
}

impl ReadyBoardTracer {
    /// Create a new instance of the `Tracer`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = ReadyBoardFlow::location();
        let metric = ReadyBoardFlow;
        let state = ReadyBoardState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Add a board
    pub fn add_board(&self, name: String, paths: HashSet<Path>) {
        let data = ReadyBoardEvent::AddBoard { name, paths };
        self.tracer.send(data, None);
    }
}
