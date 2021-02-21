use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

#[derive(Debug)]
pub enum CounterDelta {
    Increment(f64),
}

impl TracerEvent for CounterDelta {
    type Snapshot = f64;

    fn aggregate(self, snapshot: &mut Self::Snapshot) {
        match self {
            Self::Increment(delta) => {
                *snapshot = *snapshot + delta;
            }
        }
    }

    fn to_data(snapshot: &Self::Snapshot) -> RillData {
        RillData::CounterRecord { value: *snapshot }
    }
}

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut)]
pub struct CounterTracer {
    #[deref]
    #[deref_mut]
    tracer: Tracer<CounterDelta>,
}

impl CounterTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path) -> Self {
        let info = format!("{} counter", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::CounterStream,
        };
        let tracer = Tracer::new(description);
        Self { tracer }
    }

    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = CounterDelta::Increment(delta);
        self.tracer.send(data, timestamp);
    }
}
