use crate::flow::data::histogram::{HistogramEvent, HistogramState};
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::io::provider::Path;
use std::time::SystemTime;

/// Tracers `Histogram` metrics.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct HistogramTracer {
    tracer: Tracer<HistogramState>,
}

impl HistogramTracer {
    /// Creates a new tracer instance.
    pub fn new(path: Path, levels: Vec<f64>) -> Self {
        let state = HistogramState::new(levels, None);
        let tracer = Tracer::new_push(state, path).0;
        Self { tracer }
    }

    /// Add value of the histogram.
    pub fn add(&self, value: f64, timestamp: Option<SystemTime>) {
        let data = HistogramEvent::Add(value);
        self.tracer.send(data, timestamp, None);
    }
}
