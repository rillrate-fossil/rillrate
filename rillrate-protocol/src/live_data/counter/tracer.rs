use super::state::*;
use crate::base::stat_flow::StatFlowTracer;
use crate::manifest::Binded;
use rill_protocol::io::provider::EntryId;

pub struct CounterStatTracer {
    tracer: Binded<StatFlowTracer<CounterStatSpec>>,
}

impl CounterStatTracer {
    // TODO: Use `ms` here and move `realtime` paramter to the rillrate constructor
    pub fn new(group: EntryId, name: EntryId, realtime: bool) -> Self {
        let pull_ms = if realtime { None } else { Some(1_000) };
        let spec = CounterStatSpec { pull_ms };
        let path = vec![group, name].into();
        let tracer = Binded::new(StatFlowTracer::new(path, spec));
        Self { tracer }
    }

    pub fn inc(&self, delta: i64) {
        self.tracer.change(delta);
    }
}
