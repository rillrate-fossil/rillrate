use super::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct LogTracer {
    tracer: Tracer,
}

impl LogTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path, active: bool) -> Self {
        let info = format!("{} logger", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::LogStream,
        };
        let tracer = Tracer::new(description, active);
        Self { tracer }
    }

    /// Writes a message.
    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = RillData::LogRecord { message };
        self.tracer.send(data, timestamp);
    }
}
