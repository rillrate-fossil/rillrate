use super::state::*;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;
use std::collections::BTreeMap;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ListFlowTracer<T: ListFlowSpec> {
    tracer: Tracer<ListFlowState<T>>,
}

impl<T: ListFlowSpec> ListFlowTracer<T> {
    pub fn new(path: Path) -> Self {
        let state = ListFlowState::new();
        let tracer = Tracer::new_push(state, path);
        Self { tracer }
    }

    pub fn add_record(&self, id: T::Id, record: T::Record) {
        let update = ListFlowEvent::AddRecord { record };
        let msg = ListEventEnvelope::SingleRecord { id, update };
        self.tracer.send(msg, None);
    }

    pub fn update_record(&self, id: T::Id, update: impl Into<T::Update>) {
        let update = update.into();
        let update = ListFlowEvent::UpdateRecord { update };
        let msg = ListEventEnvelope::SingleRecord { id, update };
        self.tracer.send(msg, None);
    }

    pub fn remove_record(&self, id: T::Id) {
        let update = ListFlowEvent::RemoveRecord;
        let msg = ListEventEnvelope::SingleRecord { id, update };
        self.tracer.send(msg, None);
    }

    pub fn full_snapshot(&self, records: BTreeMap<T::Id, T::Record>) {
        let msg = ListEventEnvelope::FullSnapshot { records };
        self.tracer.send(msg, None);
    }

    pub fn clear(&self) {
        let msg = ListEventEnvelope::Clear;
        self.tracer.send(msg, None);
    }
}
