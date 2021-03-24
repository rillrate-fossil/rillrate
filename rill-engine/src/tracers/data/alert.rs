use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::flow::data::alert::{AlertEvent, AlertMetric, AlertState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct AlertTracer {
    tracer: Tracer<AlertMetric>,
}

impl AlertTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let metric = AlertMetric;
        let state = AlertState::new();
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Writes a message.
    pub fn alert(&self, message: String, timestamp: Option<SystemTime>) {
        let data = AlertEvent { msg: message };
        self.tracer.send(data, timestamp);
    }
}
