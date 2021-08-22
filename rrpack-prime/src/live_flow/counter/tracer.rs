use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Counter {
    tracer: BindedTracer<CounterState>,
}

impl Counter {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: CounterSpec) -> Self {
        let tracer = BindedTracer::new(auto_path, mode, spec);
        Self { tracer }
    }

    pub fn inc(&self, delta: impl Into<i64>) {
        let msg = CounterEvent::Inc {
            delta: delta.into(),
        };
        self.tracer.send(msg, None);
    }
}
