use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Clone, Default)]
pub struct HistogramOpts {
    pub levels: Vec<f64>,
}

impl From<HistogramOpts> for HistogramSpec {
    fn from(opts: HistogramOpts) -> Self {
        Self {
            levels: opts.levels,
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Histogram {
    tracer: BindedTracer<HistogramState>,
}

impl Histogram {
    pub fn new(
        auto_path: impl Into<AutoPath>,
        mode: FlowMode,
        spec: impl Into<HistogramSpec>,
    ) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), mode, spec.into());
        Self { tracer }
    }

    pub fn add(&self, value: impl Into<f64>) {
        let msg = HistogramEvent::Add(value.into());
        self.tracer.send(msg, None);
    }
}
