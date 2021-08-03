use super::state::*;
use crate::base::stat_flow::StatFlowTracer;
use crate::manifest::descriptions_list::Binded;
use rill_protocol::io::provider::EntryId;

pub struct CounterStatTracer {
    tracer: Binded<StatFlowTracer<CounterStatSpec>>,
}

impl CounterStatTracer {
    pub fn new(name: impl Into<EntryId>, realtime: bool) -> Self {
        let pull_ms = if realtime { None } else { Some(1_000) };
        let spec = CounterStatSpec {
            name: name.into(),
            pull_ms,
        };
        let tracer = Binded::new(StatFlowTracer::new(spec));
        Self { tracer }
    }

    pub fn inc(&self, delta: i64) {
        self.tracer.change(delta);
    }
}
