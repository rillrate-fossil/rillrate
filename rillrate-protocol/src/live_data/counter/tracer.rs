use super::state::*;
use crate::base::stat_flow::{StatFlowTracer, StatFlowWatcher};
use crate::manifest::descriptions_flow::DescriptionBinder;
use rill_protocol::io::provider::EntryId;

pub struct CounterStatTracer {
    tracer: StatFlowTracer<CounterStatSpec>,
    binder: DescriptionBinder,
}

impl CounterStatTracer {
    pub fn new(name: impl Into<EntryId>, realtime: bool) -> Self {
        let pull_ms = if realtime { None } else { Some(1_000) };
        let spec = CounterStatSpec {
            name: name.into(),
            pull_ms,
        };
        let tracer = StatFlowTracer::new(spec);
        let binder = DescriptionBinder::new(&tracer);
        Self { tracer, binder }
    }

    pub fn inc(&self, delta: i64) {
        self.tracer.change(delta);
    }
}
