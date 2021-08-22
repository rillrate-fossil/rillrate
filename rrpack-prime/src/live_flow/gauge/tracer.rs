use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::Binder;
use derive_more::{Deref, DerefMut};
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Gauge {
    #[deref]
    #[deref_mut]
    tracer: Tracer<GaugeState>,
    _binder: Binder,
}

impl Gauge {
    pub fn new(auto_path: impl Into<AutoPath>, mode: FlowMode, spec: GaugeSpec) -> Self {
        let path = auto_path.into();
        let state = spec.into();
        let tracer = Tracer::new(state, path.into(), mode);
        let binder = Binder::new(&tracer);
        Self {
            tracer,
            _binder: binder,
        }
    }

    pub fn set(&self, value: impl Into<f64>) {
        let msg = GaugeEvent::Set {
            value: value.into(),
        };
        self.tracer.send(msg, None);
    }
}
