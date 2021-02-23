use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType, Timestamp};
use std::time::SystemTime;

#[derive(Debug)]
pub enum GaugeUpdate {
    Increment(f64),
    Decrement(f64),
    Set(f64),
}

#[derive(Debug, Default)]
pub struct GaugeState {
    gauge: f64,
    last_event: Option<RillEvent>,
}

impl TracerEvent for GaugeUpdate {
    type State = GaugeState;

    fn aggregate(self, state: &mut Self::State, timestamp: Timestamp) -> Option<&RillEvent> {
        match self {
            Self::Increment(delta) => {
                state.gauge += delta;
            }
            Self::Decrement(delta) => {
                state.gauge -= delta;
            }
            Self::Set(value) => {
                state.gauge = value;
            }
        }
        let data = RillData::GaugeValue { value: state.gauge };
        let last_event = RillEvent { timestamp, data };
        state.last_event = Some(last_event);
        state.last_event.as_ref()
    }

    fn to_snapshot(state: &Self::State) -> Vec<RillEvent> {
        state.last_event.clone().into_iter().collect()
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
    pub fn new(path: Path) -> Self {
        let info = format!("{} gauge", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::GaugeStream,
        };
        let tracer = Tracer::new(description);
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
