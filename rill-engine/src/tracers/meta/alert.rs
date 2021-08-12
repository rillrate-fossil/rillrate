use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::alert::{AlertEvent, AlertState};
use rill_protocol::io::provider::Path;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct AlertTracer {
    tracer: Tracer<AlertState>,
}

impl AlertTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let state = AlertState::new();
        // TODO: Use the `Receiver`
        let tracer = Tracer::new_push(state, path);
        Self { tracer }
    }

    /// Writes a message.
    pub fn alert(&self, message: String) {
        let data = AlertEvent { msg: message };
        self.tracer.send(data, None);
    }
}
