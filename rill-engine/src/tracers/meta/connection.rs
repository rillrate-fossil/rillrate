use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::meta::{
    connection::{ConnectionEvent, ConnectionFlow, ConnectionState},
    MetaFlow,
};
use rill_protocol::io::provider::{EntryId, Path};

/// This tracer that informs about entries.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct ConnectionTracer {
    tracer: Tracer<ConnectionFlow>,
}

impl ConnectionTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(uid: EntryId) -> Self {
        let mut path = Path::single(uid);
        path.extend(ConnectionFlow::location());
        let metric = ConnectionFlow;
        let state = ConnectionState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Add a round trip value
    pub fn add_round_trip(&self, ms: u32) {
        let data = ConnectionEvent::AddRoundTrip { ms };
        self.tracer.send(data, None);
    }
}
