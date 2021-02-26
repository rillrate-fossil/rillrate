use super::frame::Frame;
use super::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType};
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

impl TracerState for LogState {
    type Item = LogRecord;

    fn aggregate(&mut self, items: Vec<DataEnvelope<Self::Item>>) -> Vec<RillEvent> {
        let mut records = Vec::new();
        for item in items {
            let (data, ts) = item.unpack();
            match data {
                LogRecord::Message(msg) => {
                    let data = RillData::LogRecord { message: msg };
                    let last_event = RillEvent {
                        timestamp: ts,
                        data,
                    };
                    self.frame.insert(last_event.clone());
                    records.push(last_event);
                }
            }
        }
        records
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        self.frame.iter().cloned().collect()
    }
}

impl TracerEvent for LogRecord {
    type State = LogState;
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
