use super::state::*;
use crate::auto_path::AutoPath;
use crate::base_flow::stat_flow::StatFlowTracer;
use crate::manifest::Binded;

pub struct Counter {
    tracer: Binded<StatFlowTracer<CounterSpec>>,
}

impl Counter {
    // TODO: Use `ms` here and move `realtime` paramter to the rillrate constructor
    pub fn new(auto_path: impl Into<AutoPath>, realtime: bool) -> Self {
        let path = auto_path.into();
        let pull_ms = if realtime { None } else { Some(1_000) };
        let spec = CounterSpec { pull_ms };
        let tracer = Binded::new(StatFlowTracer::new(path.into(), spec));
        Self { tracer }
    }

    pub fn inc(&self, delta: i64) {
        self.tracer.change(delta);
    }
}
