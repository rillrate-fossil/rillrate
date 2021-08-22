use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Board {
    tracer: BindedTracer<BoardState>,
}

impl Board {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: BoardSpec) -> Self {
        let tracer = BindedTracer::new(auto_path, mode, spec);
        Self { tracer }
    }

    pub fn set(&self, key: impl ToString, value: impl ToString) {
        let msg = BoardEvent::Assign {
            key: key.to_string(),
            value: value.to_string(),
        };
        self.tracer.send(msg, None);
    }

    pub fn remove(&self, key: impl ToString) {
        let msg = BoardEvent::Remove {
            key: key.to_string(),
        };
        self.tracer.send(msg, None);
    }
}
