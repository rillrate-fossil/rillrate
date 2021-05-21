use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::control::toggle::{ToggleEvent, ToggleState};
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
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Set selected value.
    pub fn set_active(&self, active: bool) {
        let event = ToggleEvent { active };
        self.tracer.send(event, None);
    }
}
