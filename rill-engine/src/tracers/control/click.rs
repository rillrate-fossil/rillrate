use crate::tracers::tracer::Tracer;
use anyhow::Error;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::click::ClickState;
use rill_protocol::io::provider::Path;

/// Receives clicks from a user.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ClickWatcher {
    tracer: Tracer<ClickState>,
}

impl ClickWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path, caption: String) -> Self {
        let state = ClickState::new(caption);
        let tracer = Tracer::new_watcher(state, path);
        Self { tracer }
    }

    /// Wait for the click event.
    pub async fn watch_click(&mut self) -> Result<(), Error> {
        self.tracer.recv().await.map(drop)
    }
}
