use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Click {
    tracer: BindedTracer<ClickState>,
}

impl Click {
    pub fn new(auto_path: impl Into<AutoPath>, spec: ClickSpec) -> Self {
        let tracer = BindedTracer::new(auto_path, FlowMode::Realtime, spec);
        Self { tracer }
    }

    pub fn apply(&self) {
        let msg = ClickEvent;
        self.tracer.send(msg, None);
    }
}
