use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType, Timestamp};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug)]
pub enum DictRecord {
    // TODO: Track hash templates here
    Association { key: String, value: String },
}

pub struct Record {
    timestamp: Timestamp,
    value: String,
}

impl TracerEvent for DictRecord {
    type State = HashMap<String, Record>;

    fn aggregate(self, state: &mut Self::State, timestamp: Timestamp) -> Option<&RillEvent> {
        match self {
            Self::Association { key, value } => {
                let record = Record { timestamp, value };
                state.insert(key, record);
                None
            }
        }
    }

    fn to_snapshot(state: &Self::State) -> Vec<RillEvent> {
        state
            .iter()
            .map(|(key, record)| {
                let data = RillData::DictRecord {
                    key: key.clone(),
                    value: record.value.clone(),
                };
                let event = RillEvent {
                    timestamp: record.timestamp.clone(),
                    data,
                };
                event
            })
            .collect()
    }
}

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct DictTracer {
    tracer: Tracer<DictRecord>,
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

    /// Writes a message.
    pub fn log(&self, key: impl ToString, value: impl ToString, timestamp: Option<SystemTime>) {
        let data = DictRecord::Association {
            key: key.to_string(),
            value: value.to_string(),
        };
        self.tracer.send(data, timestamp);
    }
}
