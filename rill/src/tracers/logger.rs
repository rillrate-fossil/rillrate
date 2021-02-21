use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

#[derive(Debug)]
pub enum LogRecord {
    // TODO: Track hash templates here
    Message(String),
}

impl TracerEvent for LogRecord {
    type Snapshot = Option<String>;

    fn aggregate(self, snapshot: &mut Self::Snapshot) {
        match self {
            Self::Message(msg) => {
                *snapshot = Some(msg);
            }
        }
    }

    fn to_data(snapshot: &Self::Snapshot) -> RillData {
        RillData::LogRecord {
            message: snapshot.clone().unwrap_or_default(),
        }
    }
}

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct LogTracer {
    tracer: Tracer<LogRecord>,
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
        let data = LogRecord::Message(message);
        self.tracer.send(data, timestamp);
    }
}
