use super::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{
    Description, DictUpdate, Path, RillData, RillEvent, StreamType, Timestamp,
};
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

impl TracerState for DictState {
    type Item = DictRecord;

    fn aggregate(&mut self, items: Vec<DataEnvelope<Self::Item>>) -> Vec<RillEvent> {
        let mut timestamp = None;
        let mut changes = HashMap::new();
        for item in items {
            let (data, ts) = item.unpack();
            match data {
                DictRecord::Association { key, value } => {
                    let record = Record {
                        timestamp: ts.clone(),
                        value: value.clone(),
                    };
                    self.map.insert(key.clone(), record);
                    changes.insert(key, value);
                }
            }
            timestamp = Some(ts);
        }
        if let Some(timestamp) = timestamp {
            let update = DictUpdate::Aggregated { map: changes };
            let data = RillData::DictUpdate(update);
            let last_event = RillEvent { timestamp, data };
            self.last_event = Some(last_event);
        }
        self.last_event.clone().into_iter().collect()
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        let (ts, map) =
            self.map
                .iter()
                .fold((None, HashMap::new()), |(_, mut map), (key, record)| {
                    map.insert(key.clone(), record.value.clone());
                    (Some(&record.timestamp), map)
                });
        if let Some(timestamp) = ts.cloned() {
            let update = DictUpdate::Aggregated { map };
            let data = RillData::DictUpdate(update);
            let event = RillEvent { timestamp, data };
            vec![event]
        } else {
            Vec::new()
        }
    }
}

impl TracerEvent for DictRecord {
    type State = DictState;
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
