use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::{
    connection::{ConnectionEvent, ConnectionFlow, ConnectionState},
    MetaFlow,
};

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ConnectionTracer {
    tracer: Tracer<ConnectionFlow>,
}

impl ConnectionTracer {
    /// Create a new instance of the `Tracer`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = ConnectionFlow::location();
        let metric = ConnectionFlow;
        let state = ConnectionState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Add a round trip value
    pub fn add_board(&self, ms: u32) {
        let data = ConnectionEvent::AddRoundTrip { ms };
        self.tracer.send(data, None);
    }
}
