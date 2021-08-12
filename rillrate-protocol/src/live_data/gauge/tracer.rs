use super::state::*;
use crate::base::stat_flow::StatFlowTracer;
use crate::live_data::auto_path::AutoPath;
use crate::manifest::Binded;

pub struct Gauge {
    tracer: Binded<StatFlowTracer<GaugeSpec>>,
}

impl Gauge {
    // TODO: Use `ms` here and move `realtime` paramter to the rillrate constructor
    pub fn new(auto_path: AutoPath, /* TODO: Expect `Spec` here. */ realtime: bool) -> Self {
        let path = auto_path.into();
        let pull_ms = if realtime { None } else { Some(1_000) };
        let spec = GaugeSpec { pull_ms };
        let tracer = Binded::new(StatFlowTracer::new(path, spec));
        Self { tracer }
    }

    pub fn set(&self, value: f64) {
        self.tracer.change(value);
    }
}
