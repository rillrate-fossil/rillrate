use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use crate::tracers::utils::Frame;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{Description, Path, RillData, RillEvent, StreamType};
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
    frame: Frame<RillEvent>,
}

impl TracerState for GaugeState {
    type Item = GaugeUpdate;

    fn aggregate(
        &mut self,
        items: Vec<DataEnvelope<Self::Item>>,
        mut outgoing: Option<&mut Vec<RillEvent>>,
    ) {
        for item in items {
            let DataEnvelope::Event {
                timestamp: ts,
                data,
            } = item;
            match data {
                GaugeUpdate::Increment(delta) => {
                    self.gauge += delta;
                }
                GaugeUpdate::Decrement(delta) => {
                    self.gauge -= delta;
                }
                GaugeUpdate::Set(value) => {
                    self.gauge = value;
                }
            }
            let data = RillData::GaugeValue { value: self.gauge };
            let last_event = RillEvent {
                timestamp: ts,
                data,
            };
            if let Some(outgoing) = outgoing.as_mut() {
                outgoing.push(last_event.clone());
            }
            self.frame.insert(last_event);
        }
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        self.frame.iter().cloned().collect()
    }
}

impl TracerEvent for GaugeUpdate {
    type State = GaugeState;
}

/// Sends metrics as `gauge` that can change value to any.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct GaugeTracer {
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
