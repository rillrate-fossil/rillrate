use super::state::*;
use crate::base::stat_flow::StatFlowTracer;
use crate::live_data::live_path::LivePath;
use crate::manifest::Binded;

pub struct CounterStatTracer {
    tracer: Binded<StatFlowTracer<CounterStatSpec>>,
}

impl CounterStatTracer {
    // TODO: Use `ms` here and move `realtime` paramter to the rillrate constructor
    pub fn new(live_path: LivePath, realtime: bool) -> Self {
        let path = live_path.into();
        let pull_ms = if realtime { None } else { Some(1_000) };
        let spec = CounterStatSpec { pull_ms };
        let tracer = Binded::new(StatFlowTracer::new(path, spec));
        Self { tracer }
    }

    pub fn inc(&self, delta: i64) {
        self.tracer.change(delta);
    }
}
