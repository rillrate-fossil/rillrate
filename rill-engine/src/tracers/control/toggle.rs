use crate::tracers::tracer::Tracer;
use anyhow::Error;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::toggle::ToggleState;
use rill_protocol::io::provider::Path;

/// Receives toggle events from a user.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ToggleWatcher {
    tracer: Tracer<ToggleState>,
}

impl ToggleWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path, caption: String, active: bool) -> Self {
        let state = ToggleState::new(caption, active);
        let tracer = Tracer::new_watcher(state, path);
        Self { tracer }
    }

    /*
    /// Wait for the toggle event.
    pub async fn watch_toggle(&mut self) -> Result<bool, Error> {
        self.tracer
            .recv()
            .await
            .map(|timed_event| timed_event.event.active)
    }
    */
}
