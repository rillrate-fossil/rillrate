use super::frame::Frame;
use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType, Timestamp};
use std::time::SystemTime;

#[derive(Debug)]
pub enum LogRecord {
    // TODO: Track hash templates here
    Message(String),
}

#[derive(Debug, Default)]
pub struct LogState {
    frame: Frame<RillEvent>,
}

impl TracerEvent for LogRecord {
    type State = LogState;

    fn aggregate(self, state: &mut Self::State, timestamp: Timestamp) -> Option<&RillEvent> {
        match self {
            Self::Message(msg) => {
                let data = RillData::LogRecord { message: msg };
                let last_event = RillEvent { timestamp, data };
                state.frame.insert(last_event)
            }
        }
    }

    fn to_snapshot(state: &Self::State) -> Vec<RillEvent> {
        state.frame.iter().cloned().collect()
    }
}

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct LogTracer {
    tracer: Tracer<LogRecord>,
}

impl LogTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let info = format!("{} logger", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::LogStream,
        };
        let tracer = Tracer::new(description);
        Self { tracer }
    }

    /// Writes a message.
    pub fn log(&self, message: String, timestamp: Option<SystemTime>) {
        let data = LogRecord::Message(message);
        self.tracer.send(data, timestamp);
    }
}
