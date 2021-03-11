use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::dict::{DictEvent, DictState};
use rill_protocol::io::provider::{Description, Path, StreamType};
use std::time::SystemTime;

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct DictTracer {
    tracer: Tracer<DictState>,
}

impl DictTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let info = format!("{} dictionary", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::DictStream,
        };
        let tracer = Tracer::new(description);
        Self { tracer }
    }

    /// Set a value to key.
    pub fn set(&self, key: impl ToString, value: impl ToString, timestamp: Option<SystemTime>) {
        let data = DictEvent::SetValue {
            key: key.to_string(),
            value: value.to_string(),
        };
        self.tracer.send(data, timestamp);
    }
}
