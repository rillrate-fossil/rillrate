use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Gauge {
    tracer: BindedTracer<GaugeState>,
}

impl Gauge {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: GaugeSpec) -> Self {
        let tracer = BindedTracer::new(auto_path, mode, spec);
        Self { tracer }
    }

    pub fn set(&self, value: impl Into<f64>) {
        let msg = GaugeEvent::Set {
            value: value.into(),
        };
        self.tracer.send(msg, None);
    }
}
