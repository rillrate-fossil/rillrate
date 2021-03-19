use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::pulse::{PulseEvent, PulseMetric, PulseState};
use rill_protocol::io::provider::Path;
use rill_protocol::range::Range;
use std::time::SystemTime;

/// Tracers `Gauge` metrics.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct GaugeTracer {
    tracer: Tracer<PulseMetric>,
}

impl GaugeTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path, min: f64, max: f64) -> Self {
        let range = Range::new(min, max);
        let metric = PulseMetric::Gauge { range };
        let state = PulseState::new(None);
        let tracer = Tracer::new(metric, state, path, None);
        Self { tracer }
    }

    /// Set value of the gauge.
    pub fn set(&self, value: f64, timestamp: Option<SystemTime>) {
        let data = PulseEvent::Set(value);
        self.tracer.send(data, timestamp);
    }
}
