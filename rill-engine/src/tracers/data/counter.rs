use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::data::counter::CounterEvent;
use rill_protocol::io::provider::{Description, Path, RillData, RillEvent, StreamType};
use std::time::SystemTime;

/*
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

    fn aggregate(
        &mut self,
        items: Vec<DataEnvelope<Self::Item>>,
        outgoing: Option<&mut Vec<RillEvent>>,
    ) {
        let mut timestamp = None;
        for item in items {
            let DataEnvelope::Event {
                timestamp: ts,
                data,
            } = item;
            match data {
                CounterDelta::Increment(delta) => {
                    self.counter += delta;
                }
            }
            timestamp = Some(ts);
        }
        if let Some(timestamp) = timestamp {
            let data = RillData::CounterRecord {
                value: self.counter,
            };
            let last_event = RillEvent { timestamp, data };
            if let Some(outgoing) = outgoing {
                outgoing.push(last_event.clone());
            }
            self.last_event = Some(last_event);
        }
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        self.last_event.clone().into_iter().collect()
    }
}

impl TracerEvent for CounterDelta {
    type State = CounterState<Item = Self>;
}
*/

/// Tracers `Counter` metrics that can increments only.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CounterTracer {
    tracer: Tracer<CounterEvent>,
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
        let data = CounterEvent { increment: delta };
        self.tracer.send(data, timestamp);
    }
}
