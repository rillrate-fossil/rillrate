use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillEvent, StreamType};
use std::time::SystemTime;
use std::collections::HashMap;

#[derive(Debug)]
pub enum DictRecord {
    // TODO: Track hash templates here
    Association {
        key: String,
        value: String,
    },
}

impl TracerEvent for DictRecord {
    type State = HashMap<String, String>;

    fn aggregate(self, state: &mut Self::State, timestamp: Timestamp) -> Option<&RillEvent> {
        match self {
            Self::Association { key, value } => {
                state.insert(key, value);
            }
            None
        }
    }

    fn to_deltas(state: &Self::State) -> Vec<RillEvent> {
        state.iter().map(|(key, value)| {
            RillEvent::DictRecord {
                key: key.clone(),
                value: value.clone(),
            }
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

