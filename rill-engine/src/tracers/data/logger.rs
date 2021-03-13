use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::logger::{LogEvent, LogMetric, LogState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LogTracer {
    tracer: Tracer<LogMetric>,
}

impl LogTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let state = LogState::new();
        let tracer = Tracer::new(state, path, None);
        Self { tracer }
    }

    /// Writes a message.
    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = LogEvent { msg: message };
        self.tracer.send(data, timestamp);
    }
}
