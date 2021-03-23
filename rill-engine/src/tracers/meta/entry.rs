use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::io::provider::{EntryId, Path};
use rill_protocol::metadata::entry::{EntryEvent, EntryMetric, EntryState};

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct EntryTracer {
    tracer: Tracer<EntryMetric>,
}

impl EntryTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let metric = EntryMetric;
        let state = EntryState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Set a value to key.
    pub fn add(&self, name: EntryId) -> ProviderEntry {
        ProviderEntry::new(self.tracer.clone(), name)
    }
}

pub struct ProviderEntry {
    tracer: Tracer<EntryMetric>,
    name: EntryId,
}

impl ProviderEntry {
    fn new(tracer: Tracer<EntryMetric>, name: EntryId) -> Self {
        let this = Self { tracer, name };
        this.register();
        this
    }

    fn register(&self) {
        let data = EntryEvent::AddProvider {
            name: self.name.clone(),
        };
        self.tracer.send(data, None);
    }

    fn unregister(&self) {
        let data = EntryEvent::RemoveProvider {
            name: self.name.clone(),
        };
        self.tracer.send(data, None);
    }
}

impl Drop for ProviderEntry {
    fn drop(&mut self) {
        self.unregister();
    }
}
