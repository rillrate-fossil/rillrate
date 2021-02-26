use super::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType};
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

impl TracerState for CounterState {
    type Item = CounterDelta;

    fn aggregate(&mut self, items: Vec<DataEnvelope<Self::Item>>) -> Option<&RillEvent> {
        let mut timestamp = None;
        for item in items {
            let (data, ts) = item.unpack();
            match data {
                CounterDelta::Increment(delta) => {
                    self.counter += delta;
                }
            }
            timestamp = Some(ts);
        }
        let timestamp = timestamp?;
        let data = RillData::CounterRecord {
            value: self.counter,
        };
        let last_event = RillEvent { timestamp, data };
        self.last_event = Some(last_event);
        self.last_event.as_ref()
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        self.last_event.clone().into_iter().collect()
    }
}

impl TracerEvent for CounterDelta {
    type State = CounterState<Item = Self>;
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
