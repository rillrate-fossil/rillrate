use crate::tracers::tracer::Tracer;
use anyhow::Error;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::toggle::{ToggleFlow, ToggleState};
use rill_protocol::io::provider::Path;

/// Receives toggle events from a user.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ToggleWatcher {
    tracer: Tracer<ToggleFlow>,
}

impl ToggleWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path, caption: String, active: bool) -> Self {
        let flow = ToggleFlow;
        let state = ToggleState::new(caption, active);
        let tracer = Tracer::new_watcher(flow, state, path);
        Self { tracer }
    }

    /// Wait for the toggle event.
    pub async fn watch_toggle(&mut self) -> Result<bool, Error> {
        self.tracer
            .recv()
            .await
            .map(|timed_event| timed_event.event.active)
    }
}
