use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;
use rrpack_basis::{AutoPath, BindedTracer};

#[derive(TracerOpts, Clone, Default)]
pub struct BoardOpts {}

impl From<BoardOpts> for BoardSpec {
    fn from(_opts: BoardOpts) -> Self {
        Self {}
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Board {
    tracer: BindedTracer<BoardState>,
}

impl Board {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: impl Into<BoardSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
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
