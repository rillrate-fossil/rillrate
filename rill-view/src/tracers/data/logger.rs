use crate::flow::data::logger::{LoggerEvent, LoggerState};
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LoggerTracer {
    tracer: Tracer<LoggerState>,
}

impl LoggerTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let state = LoggerState::new();
        let tracer = Tracer::new_push(state, path).0;
        Self { tracer }
    }

    /// Writes a message.
    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = LoggerEvent { msg: message };
        self.tracer.send(data, timestamp, None);
    }
}
