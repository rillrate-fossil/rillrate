use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Histogram {
    tracer: BindedTracer<HistogramState>,
}

impl Histogram {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: HistogramSpec) -> Self {
        let tracer = BindedTracer::new(auto_path, mode, spec);
        Self { tracer }
    }

    pub fn add(&self, value: f64) {
        let msg = HistogramEvent::Add(value);
        self.tracer.send(msg, None);
    }
}
