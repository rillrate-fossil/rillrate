use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, StreamType};
use std::time::SystemTime;

#[derive(Debug)]
pub enum GaugeUpdate {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}

impl TracerEvent for GaugeUpdate {
    type Snapshot = f64;

    fn aggregate(self, snapshot: &mut Self::Snapshot) {
        match self {
            Self::Increment(delta) => {
                *snapshot = *snapshot + delta;
            }
            Self::Decrement(delta) => {
                *snapshot = *snapshot - delta;
            }
            Self::Set(value) => {
                *snapshot = value;
            }
        }
    }

    fn to_data(snapshot: &Self::Snapshot) -> RillData {
        RillData::GaugeValue {
            value: *snapshot,
        }
    }
}


/// Sends metrics as `gauge` that can change value to any.
#[derive(Debug, Deref, DerefMut)]
pub struct GaugeTracer {
    #[deref]
    #[deref_mut]
    tracer: Tracer<GaugeUpdate>,
}

impl GaugeTracer {
    /// Creates a new `Gauge` tracer.
    pub fn new(path: Path, active: bool) -> Self {
        let info = format!("{} gauge", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::GaugeStream,
        };
        let tracer = Tracer::new(description, active);
        Self { tracer }
    }

    /// Increments the value by the specific delta.
    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = GaugeUpdate::Increment(delta);
        self.tracer.send(data, timestamp);
    }

    /// Decrements the value by the specific delta.
    pub fn dec(&self, delta: f64, timestamp: Option<SystemTime>) {
        let data = GaugeUpdate::Decrement(delta);
        self.tracer.send(data, timestamp);
    }

    /// Set the value.
    pub fn set(&self, new_value: f64, timestamp: Option<SystemTime>) {
        let data = GaugeUpdate::Set(new_value);
        self.tracer.send(data, timestamp);
    }
}
