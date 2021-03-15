use crate::tracers::tracer::Tracer;
use derive_more::{Deref, DerefMut};
use rill_protocol::data::histogram::{HistogramEvent, HistogramMetric, HistogramState};
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Tracers `Histogram` metrics.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct HistogramTracer {
    tracer: Tracer<HistogramMetric>,
}

impl HistogramTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path, levels: &[f64]) -> Self {
        let state = HistogramState::new(levels, None);
        let tracer = Tracer::new(state, path, None);
        Self { tracer }
    }

    /// Add value of the histogram.
    pub fn add(&self, value: f64, timestamp: Option<SystemTime>) {
        let data = HistogramEvent::Add(value);
        self.tracer.send(data, timestamp);
    }
}
