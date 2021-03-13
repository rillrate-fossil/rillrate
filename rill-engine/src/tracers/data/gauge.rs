use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::gauge::{GaugeEvent, GaugeMetric, GaugeState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Tracers `Gauge` metrics that can increments only.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct GaugeTracer {
    #[deref]
    #[deref_mut]
    tracer: Tracer<GaugeMetric>,
    min: f64,
    max: f64,
}

impl GaugeTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path, mut min: f64, mut max: f64) -> Self {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        let state = GaugeState::new(min, max);
        let tracer = Tracer::new(state, path, None);
        Self { tracer, min, max }
    }

    /// Set value of the gauge.
    pub fn set(&self, value: f64, timestamp: Option<SystemTime>) {
        let value = {
            if value < self.min {
                self.min
            } else if value > self.max {
                self.max
            } else {
                value
            }
        };
        let data = GaugeEvent::Set(value);
        self.tracer.send(data, timestamp);
    }
}
