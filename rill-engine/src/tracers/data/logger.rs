use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::data::logger::LogEvent;
use rill_protocol::frame::Frame;
use rill_protocol::io::provider::{Description, Path, RillData, RillEvent, StreamType};
use std::time::SystemTime;

/*
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

    fn aggregate(
        &mut self,
        items: Vec<DataEnvelope<Self::Item>>,
        mut outgoing: Option<&mut Vec<RillEvent>>,
    ) {
        for item in items {
            let DataEnvelope::Event {
                timestamp: ts,
                data,
            } = item;
            match data {
                LogRecord::Message(msg) => {
                    let data = RillData::LogRecord {
                        message: msg.to_owned(),
                    };
                    let last_event = RillEvent {
                        timestamp: ts,
                        data,
                    };
                    if let Some(outgoing) = outgoing.as_mut() {
                        outgoing.push(last_event.clone());
                    }
                    self.frame.insert(last_event);
                }
            }
        }
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        self.frame.iter().cloned().collect()
    }
}

impl TracerEvent for LogRecord {
    type State = LogState;
}
*/

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct LogTracer {
    tracer: Tracer<LogEvent>,
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
        let data = LogEvent { msg: message };
        self.tracer.send(data, timestamp);
    }
}
