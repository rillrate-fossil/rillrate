use crate::tracers::tracer::Tracer;
use anyhow::Error;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::selector::SelectorState;
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
        let tracer = Tracer::new_watcher(state, path);
        Self { tracer }
    }

    /*
    /// Wait for the select event.
    pub async fn watch_select(&mut self) -> Result<String, Error> {
        // TODO: Use cloneable values of type `K` and keep them in an `Arc`
        self.tracer
            .recv()
            .await
            .map(|timed_event| timed_event.event.select)
    }
    */
}
