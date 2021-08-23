use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Slider {
    tracer: BindedTracer<SliderState>,
}

impl Slider {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<SliderSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self, set_value: f64) {
        let msg = SliderEvent { set_value };
        self.tracer.send(msg, None);
    }
}
