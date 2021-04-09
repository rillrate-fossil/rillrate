use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::connection::{ConnectionEvent, ConnectionState};
use rill_protocol::io::provider::Path;

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ConnectionTracer {
    tracer: Tracer<ConnectionState>,
}

impl ConnectionTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let state = ConnectionState::new();
        let tracer = Tracer::new_tracer(state, path, None);
        Self { tracer }
    }

    /// Add a round trip value
    pub fn add_round_trip(&self, ms: u32) {
        let data = ConnectionEvent::AddRoundTrip { ms };
        self.tracer.send(data, None);
    }
}
