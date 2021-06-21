use crate::flow::control::click::{ClickEvent, ClickState};
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
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
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Send `Clicked` event.
    pub fn clicked(&self) {
        let event = ClickEvent;
        self.tracer.send(event, None, None);
    }
}
