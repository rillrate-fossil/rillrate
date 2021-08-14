use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Histogram {
    #[deref]
    #[deref_mut]
    tracer: Tracer<HistogramState>,
    _binder: Binder,
}

impl Histogram {
    pub fn new(auto_path: impl Into<AutoPath>, levels: Vec<f64>) -> Self {
        let path = auto_path.into();
        let state = HistogramState::new(levels);
        let tracer = Tracer::new(state, path.into(), None, None);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn add(&self, value: f64) {
        let msg = HistogramEvent::Add(value);
        self.tracer.send(msg, None);
    }
}
