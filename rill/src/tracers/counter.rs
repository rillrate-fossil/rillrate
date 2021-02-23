use super::tracer::{Tracer, TracerEvent};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType, Timestamp};
use std::time::SystemTime;

#[derive(Debug)]
pub enum CounterDelta {
    Increment(f64),
}

#[derive(Debug, Default)]
pub struct CounterState {
    counter: f64,
    last_event: Option<RillEvent>,
}

impl TracerEvent for CounterDelta {
    type State = CounterState;

    fn aggregate(self, state: &mut Self::State, timestamp: Timestamp) -> Option<&RillEvent> {
        match self {
            Self::Increment(delta) => {
                state.counter += delta;
                let data = RillData::CounterRecord {
                    value: state.counter,
                };
                let last_event = RillEvent { timestamp, data };
                state.last_event = Some(last_event);
                state.last_event.as_ref()
            }
        }
    }

    fn to_snapshot(state: &Self::State) -> Vec<RillEvent> {
        state.last_event.clone().into_iter().collect()
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
