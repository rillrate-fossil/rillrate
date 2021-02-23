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

#[derive(Debug)]
pub struct Record {
    timestamp: Timestamp,
    value: String,
}

#[derive(Debug, Default)]
pub struct DictState {
    map: HashMap<String, Record>,
    last_event: Option<RillEvent>,
}

impl TracerEvent for DictRecord {
    type State = DictState;

    fn aggregate(self, state: &mut Self::State, timestamp: Timestamp) -> Option<&RillEvent> {
        match self {
            Self::Association { key, value } => {
                let record = Record {
                    timestamp: timestamp.clone(),
                    value: value.clone(),
                };
                state.map.insert(key.clone(), record);
                let data = RillData::DictRecord { key, value };
                let last_event = RillEvent { timestamp, data };
                state.last_event = Some(last_event);
                state.last_event.as_ref()
            }
        }
    }

    fn to_snapshot(state: &Self::State) -> Vec<RillEvent> {
        state
            .map
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

    /// Set a value to key.
    pub fn set(&self, key: impl ToString, value: impl ToString, timestamp: Option<SystemTime>) {
        let data = DictRecord::Association {
            key: key.to_string(),
            value: value.to_string(),
        };
        self.tracer.send(data, timestamp);
    }
}
