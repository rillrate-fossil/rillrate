use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::counter::CounterEvent;
use rill_protocol::io::provider::{Description, Path, StreamType};
use std::time::SystemTime;

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CounterTracer {
    tracer: Tracer<CounterEvent>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path) -> Self {
        let info = format!("{} counter", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::CounterStream,
        };
        let tracer = Tracer::new(description);
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = CounterEvent::Increment(delta);
        self.tracer.send(data, timestamp);
    }
}
