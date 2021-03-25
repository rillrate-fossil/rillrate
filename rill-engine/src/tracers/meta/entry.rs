use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::{
    entry::{EntryEvent, EntryFlow, EntryState},
    MetaFlow,
};
use rill_protocol::io::provider::EntryId;

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct EntryTracer {
    tracer: Tracer<EntryFlow>,
}

impl EntryTracer {
    /// Create a new instance of the `Tracer`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = EntryFlow::location();
        let metric = EntryFlow;
        let state = EntryState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Add an entry
    pub fn add(&self, name: EntryId) {
        let data = EntryEvent::AddEntry { name };
        self.tracer.send(data, None);
    }

    /// Remove an entry
    pub fn del(&self, name: EntryId) {
        let data = EntryEvent::RemoveEntry { name };
        self.tracer.send(data, None);
    }
}
