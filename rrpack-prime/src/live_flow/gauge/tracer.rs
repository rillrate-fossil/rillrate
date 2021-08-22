use super::state::*;
use crate::auto_path::AutoPath;
use crate::base_flow::stat_flow::StatFlowTracer;
use crate::manifest::Binded;
use rill_protocol::flow::core::FlowMode;

pub struct Gauge {
    tracer: Binded<StatFlowTracer<GaugeSpec>>,
}

impl Gauge {
    // TODO: Use `ms` here and move `realtime` paramter to the rillrate constructor
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: GaugeSpec) -> Self {
        let path = auto_path.into();
        let tracer = Binded::new(StatFlowTracer::new(path.into(), mode, spec));
        Self { tracer }
    }

    // TODO: Add `new_auto`
    /*
    pub fn new_auto(&str) -> Self;
    */

    pub fn set(&self, value: f64) {
        self.tracer.change(value);
    }
}
