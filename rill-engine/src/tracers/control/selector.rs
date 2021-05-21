use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::selector::{SelectorEvent, SelectorState};
use rill_protocol::io::provider::Path;

/// Receives select events from a user.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct SelectorWatcher {
    tracer: Tracer<SelectorState>,
}

impl SelectorWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path, label: String, options: Vec<String>, selected: String) -> Self {
        let state = SelectorState::new(label, options, selected);
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Set selected value.
    pub fn selected(&self, selected: String) {
        let event = SelectorEvent { selected };
        self.tracer.send(event, None);
    }
}
