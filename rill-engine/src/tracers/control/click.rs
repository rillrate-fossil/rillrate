use crate::tracers::tracer::Tracer;
use rill_protocol::flow::control::click::{ClickEvent, ClickFlow, ClickState};
use rill_protocol::io::provider::Path;

/// Receives clicks from a user.
pub struct ClickWatcher {
    tracer: Tracer<ClickFlow>,
}

impl ClickWatcher {
    /// Create a new instance of the `Watcher`.
    pub fn new(path: Path) -> Self {
        let metric = ClickFlow;
        let state = ClickState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }
}
